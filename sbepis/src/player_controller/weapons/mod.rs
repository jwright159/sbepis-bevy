use bevy::color::palettes::css;
use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::entity::{EntityKilled, GelViscosity};
use crate::fray::FrayMusic;
use crate::util::QuaternionEx;

pub mod hammer;
//pub mod rifle;
//pub mod sword;

#[derive(Event)]
pub struct EntityDamaged {
	pub victim: Entity,
	pub damage: f32,
	pub fray_modifier: f32,
}

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
}

#[derive(Component)]
pub struct EndDamageSweep;

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct SweepPivot {
	pub sweeper_length: f32,
	pub sweep_depth: f32,
	pub sweep_height: f32,
}

impl DamageSweep {
	pub fn new(transform: GlobalTransform, pivot: Entity, allies: EntityHashSet) -> Self {
		Self {
			hit_entities: EntityHashSet::default(),
			last_transform: transform,
			pivot,
			allies,
		}
	}
}

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct DebugColliderVisualizer;

#[derive(Component)]
pub struct WeaponAnimation(AnimationNodeIndex);

pub fn attack(mut weapons: Query<(&WeaponAnimation, &mut AnimationPlayer), With<ActiveWeapon>>) {
	for (animation, mut animation_player) in weapons.iter_mut() {
		let animation = animation_player.play(animation.0);
		if animation.is_finished() {
			animation.replay();
		}
	}
}

pub fn correct_animation_speed(
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

pub fn sweep_dealers(
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
				if !dealer.allies.contains(&hit_entity) {
					dealer.hit_entities.insert(hit_entity);
				}
				true
			},
		);
		commands
			.entity(debug_collider_visualizer)
			.insert(collider)
			.insert(Transform::from_translation(position).with_rotation(rotation));

		dealer.last_transform = *transform;

		if end.is_some() {
			commands
				.entity(dealer_entity)
				.remove::<DamageSweep>()
				.remove::<EndDamageSweep>();
		}
	}
}

pub fn deal_all_damage(
	mut ev_hit: EventReader<EntityDamaged>,
	mut ev_kill: EventWriter<EntityKilled>,
	mut healths: Query<&mut GelViscosity>,
) {
	for event in ev_hit.read() {
		let Ok(mut health) = healths.get_mut(event.victim) else {
			continue;
		};
		let damage = event.damage;

		if damage > 0.0 && health.value <= 0.0 {
			ev_kill.send(EntityKilled {
				entity: event.victim,
			});
		}

		health.value -= damage;
	}
}

pub fn update_damage_numbers(
	mut ev_hit: EventReader<EntityDamaged>,
	mut damage_numbers: Query<Entity, With<DamageNumbers>>,
	hit_object: Query<&Name, With<GelViscosity>>,
	mut commands: Commands,
) {
	for event in ev_hit.read() {
		let Ok(hit_object_name) = hit_object.get(event.victim) else {
			continue;
		};

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

pub fn initialize_weapon_sets(
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

pub fn switch_weapon_next(mut commands: Commands, mut weapon_sets: Query<&mut WeaponSet>) {
	for mut weapon_set in weapon_sets.iter_mut() {
		let old_weapon = weapon_set.weapons[weapon_set.active_weapon];
		hide_weapon(&mut commands, old_weapon);
		weapon_set.active_weapon = (weapon_set.active_weapon + 1) % weapon_set.weapons.len();
		let new_weapon = weapon_set.weapons[weapon_set.active_weapon];
		show_weapon(&mut commands, new_weapon);
	}
}

pub fn switch_weapon_prev(mut commands: Commands, mut weapon_sets: Query<&mut WeaponSet>) {
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
