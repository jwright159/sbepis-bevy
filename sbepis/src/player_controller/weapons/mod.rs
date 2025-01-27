use bevy::color::palettes::css;
use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;

use crate::entity::{EntityKilled, EntityKilledSet, GelViscosity};
use crate::fray::FrayMusic;
use crate::input::button_just_pressed;
use crate::player_controller::{PlayerAction, PlayerControllerPlugin};
use crate::util::{find_in_ancestors, QuaternionEx};

pub mod hammer;
pub mod rifle;
pub mod sword;

#[derive(Event)]
#[event(plugin = PlayerControllerPlugin)]
pub struct EntityHit {
	pub victim: Entity,
	pub perpetrator: Entity,
	pub allies: EntityHashSet,
	pub damage: f32,
	pub fray_modifier: f32,
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityHitSet;

#[derive(Event)]
#[event(plugin = PlayerControllerPlugin)]
pub struct EntityDamaged {
	pub victim: Entity,
	pub damage: f32,
	pub fray_modifier: f32,
}
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EntityDamagedSet;

#[derive(Component)]
pub struct DamageNumbers;

#[derive(Component)]
pub struct WeaponSet {
	pub weapons: Vec<Entity>,
	pub active_weapon: usize,
}

#[derive(Component)]
pub struct UninitializedWeaponSet;

#[derive(Component)]
pub struct ActiveWeapon;

#[derive(Component)]
pub struct DamageSweep {
	pub hit_entities: EntityHashSet,
	pub last_transform: GlobalTransform,
	pub pivot: Entity,
	pub allies: EntityHashSet,
	pub owner: Entity,
}

#[derive(Component)]
pub struct EndDamageSweep {
	pub damage: f32,
	pub fray_modifier: f32,
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct SweepPivot {
	pub sweeper_length: f32,
	pub sweep_depth: f32,
	pub sweep_height: f32,
}

impl DamageSweep {
	pub fn new(
		transform: GlobalTransform,
		pivot: Entity,
		allies: EntityHashSet,
		owner: Entity,
	) -> Self {
		Self {
			hit_entities: EntityHashSet::default(),
			last_transform: transform,
			pivot,
			allies,
			owner,
		}
	}
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct DebugColliderVisualizer;

#[derive(Component)]
pub struct WeaponAnimation(pub AnimationNodeIndex);

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::Use),
)]
fn attack(mut weapons: Query<(&WeaponAnimation, &mut AnimationPlayer), With<ActiveWeapon>>) {
	for (animation, mut animation_player) in weapons.iter_mut() {
		if let Some(animation) = animation_player.animation_mut(animation.0) {
			if animation.is_finished() {
				animation.replay();
			}
		} else {
			animation_player.stop_all();
			animation_player.play(animation.0);
		}
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
)]
fn correct_animation_speed(
	fray_music: Query<&FrayMusic>,
	mut weapons: Query<(&WeaponAnimation, &mut AnimationPlayer)>,
) {
	let fray_music = fray_music.single();
	for (animation, mut animation_player) in weapons.iter_mut() {
		if let Some(animation) = animation_player.animation_mut(animation.0) {
			animation.set_speed(fray_music.speed());
		}
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	in_set = EntityHitSet,
)]
fn sweep_dealers(
	mut commands: Commands,
	mut dealers: Query<(
		Entity,
		&mut DamageSweep,
		Option<&EndDamageSweep>,
		&GlobalTransform,
	)>,
	pivots: Query<(&SweepPivot, &GlobalTransform), Without<DamageSweep>>,
	rapier_context: Query<&RapierContext>,
	debug_collider_visualizers: Query<Entity, With<DebugColliderVisualizer>>,
	mut ev_hit: EventWriter<EntityHit>,
) {
	let debug_collider_visualizer = debug_collider_visualizers.single();
	let rapier_context = rapier_context.single();
	for (dealer_entity, mut dealer, end, transform) in dealers.iter_mut() {
		let (pivot, pivot_transform) = pivots.get(dealer.pivot).expect("Sweep pivot not found");

		let start_tip = dealer
			.last_transform
			.transform_point(pivot.sweeper_length * 0.5 * Vec3::Z);
		let end_tip = transform.transform_point(pivot.sweeper_length * 0.5 * Vec3::NEG_Z);
		let delta = end_tip - start_tip;
		let position = (end_tip + start_tip) * 0.5;

		let pivot_position = pivot_transform.translation();
		let up = (end_tip - pivot_position).cross(start_tip - pivot_position);
		let rotation = Quat::from_look_to(delta, up);

		let collider = Collider::cuboid(
			pivot.sweep_depth * 0.5,
			pivot.sweep_height * 0.5,
			delta.length() * 0.5,
		);
		rapier_context.intersections_with_shape(
			position,
			rotation,
			&collider,
			QueryFilter::new(),
			|hit_entity| {
				dealer.hit_entities.insert(hit_entity);
				true
			},
		);
		commands
			.entity(debug_collider_visualizer)
			.insert(collider)
			.insert(Transform::from_translation(position).with_rotation(rotation));

		dealer.last_transform = *transform;

		if let Some(end) = end {
			for entity in dealer.hit_entities.iter() {
				ev_hit.send(EntityHit {
					victim: *entity,
					perpetrator: dealer.owner,
					allies: dealer.allies.clone(),
					damage: end.damage,
					fray_modifier: end.fray_modifier,
				});
			}

			commands
				.entity(dealer_entity)
				.remove::<DamageSweep>()
				.remove::<EndDamageSweep>();
		}
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = EntityHitSet,
	in_set = EntityDamagedSet,
)]
fn hit_to_damage(
	parents: Query<&Parent>,
	healths: Query<Entity, With<GelViscosity>>,
	mut ev_hit: EventReader<EntityHit>,
	mut ev_damage: EventWriter<EntityDamaged>,
) {
	for event in ev_hit.read() {
		let victim = find_in_ancestors(event.victim, &healths, &parents).unwrap_or(event.victim);
		if !event.allies.contains(&victim) {
			ev_damage.send(EntityDamaged {
				victim,
				damage: event.damage,
				fray_modifier: event.fray_modifier,
			});
		}
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = EntityDamagedSet,
	in_set = EntityKilledSet,
)]
fn deal_all_damage(
	mut ev_hit: EventReader<EntityDamaged>,
	mut ev_kill: EventWriter<EntityKilled>,
	mut healths: Query<&mut GelViscosity>,
) {
	for event in ev_hit.read() {
		if let Ok(mut health) = healths.get_mut(event.victim) {
			let damage = event.damage;

			if damage > 0.0 && health.value <= 0.0 {
				ev_kill.send(EntityKilled(event.victim));
			}

			health.value -= damage;
		}
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	after = EntityDamagedSet,
)]
fn update_damage_numbers(
	mut ev_hit: EventReader<EntityDamaged>,
	mut damage_numbers: Query<Entity, With<DamageNumbers>>,
	hit_object: Query<Option<&Name>, With<GelViscosity>>,
	mut commands: Commands,
) {
	for event in ev_hit.read() {
		let Ok(hit_object_name) = hit_object.get(event.victim) else {
			continue;
		};
		let hit_object_name = hit_object_name
			.map(|name| name.as_str())
			.unwrap_or("Object");

		let damage = event.damage;
		let fray_modifier = event.fray_modifier;
		for damage_numbers in damage_numbers.iter_mut() {
			commands
				.spawn((
					TextSpan(format!("\n{hit_object_name}: {damage:.2}")),
					TextColor(Color::mix(
						&Color::from(css::RED),
						&Color::from(css::GREEN),
						fray_modifier.clamp(0.0, 1.0),
					)),
				))
				.set_parent(damage_numbers);
		}
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
)]
fn initialize_weapon_sets(
	mut commands: Commands,
	weapon_sets: Query<(Entity, &WeaponSet), With<UninitializedWeaponSet>>,
) {
	for (entity, weapon_set) in weapon_sets.iter() {
		for (index, weapon) in weapon_set.weapons.iter().enumerate() {
			if index == weapon_set.active_weapon {
				show_weapon(&mut commands, *weapon);
			} else {
				hide_weapon(&mut commands, *weapon);
			}
		}
		commands.entity(entity).remove::<UninitializedWeaponSet>();
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::NextWeapon),
)]
fn switch_weapon_next(mut commands: Commands, mut weapon_sets: Query<&mut WeaponSet>) {
	for mut weapon_set in weapon_sets.iter_mut() {
		let old_weapon = weapon_set.weapons[weapon_set.active_weapon];
		hide_weapon(&mut commands, old_weapon);
		weapon_set.active_weapon = (weapon_set.active_weapon + 1) % weapon_set.weapons.len();
		let new_weapon = weapon_set.weapons[weapon_set.active_weapon];
		show_weapon(&mut commands, new_weapon);
	}
}

#[system(
	plugin = PlayerControllerPlugin, schedule = Update,
	run_if = button_just_pressed(PlayerAction::PrevWeapon),
)]
fn switch_weapon_prev(mut commands: Commands, mut weapon_sets: Query<&mut WeaponSet>) {
	for mut weapon_set in weapon_sets.iter_mut() {
		let old_weapon = weapon_set.weapons[weapon_set.active_weapon];
		hide_weapon(&mut commands, old_weapon);
		weapon_set.active_weapon =
			(weapon_set.active_weapon + weapon_set.weapons.len() - 1) % weapon_set.weapons.len();
		let new_weapon = weapon_set.weapons[weapon_set.active_weapon];
		show_weapon(&mut commands, new_weapon);
	}
}

fn hide_weapon(commands: &mut Commands, weapon: Entity) {
	commands
		.entity(weapon)
		.remove::<ActiveWeapon>()
		.insert(Visibility::Hidden);
}

fn show_weapon(commands: &mut Commands, weapon: Entity) {
	commands
		.entity(weapon)
		.insert(ActiveWeapon)
		.insert(Visibility::Inherited);
}
