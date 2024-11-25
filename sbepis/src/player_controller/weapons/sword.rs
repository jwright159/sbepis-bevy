use std::f32::consts::PI;
use std::time::Duration;

use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;
use interpolation::EaseFunction;

use crate::fray::FrayMusic;
use crate::util::MapRange;
use crate::{gridbox_material, ok_or_continue};

use super::{DamageSweep, EndDamageSweep, EntityDamaged, InAnimation, SweepPivot};

#[derive(Component)]
pub struct SwordPivot;

#[derive(Component)]
pub struct Sword {
	pub damage: f32,
	pub pivot: Entity,
	pub allies: EntityHashSet,
	pub start_slash_time: Duration,
	side: SwordSide,
	pub follow_through_time: f32,
}

impl Sword {
	pub fn new(
		damage: f32,
		pivot: Entity,
		allies: EntityHashSet,
		follow_through_time: f32,
	) -> Self {
		Self {
			damage,
			pivot,
			allies,
			start_slash_time: Duration::ZERO,
			side: SwordSide::Left,
			follow_through_time,
		}
	}
}

enum SwordSide {
	Left,
	Right,
}

impl SwordSide {
	fn other_side(&self) -> Self {
		match self {
			SwordSide::Left => SwordSide::Right,
			SwordSide::Right => SwordSide::Left,
		}
	}

	fn angle(&self) -> f32 {
		match self {
			SwordSide::Left => -PI * 0.5,
			SwordSide::Right => PI * 0.5,
		}
	}
}

pub fn spawn_sword(
	commands: &mut Commands,
	asset_server: &AssetServer,
	materials: &mut Assets<StandardMaterial>,
	meshes: &mut Assets<Mesh>,
	body: Entity,
) -> (Entity, Entity) {
	let sword_pivot = commands
		.spawn((
			Name::new("Sword Pivot"),
			SpatialBundle::from_transform(
				Transform::from_translation(Vec3::ZERO)
					.with_rotation(Quat::from_rotation_y(-PI * 0.5)),
			),
			SwordPivot,
			SweepPivot {
				sweeper_length: 0.2,
				sweep_depth: 0.5,
				sweep_height: 0.2,
			},
		))
		.set_parent(body)
		.id();

	let sword_blade = commands
		.spawn((
			Name::new("Sword Blade"),
			PbrBundle {
				transform: Transform::default()
					.with_translation(Vec3::NEG_Z * 1.)
					.with_rotation(Quat::from_rotation_x(PI / 2.)),
				mesh: meshes.add(
					Capsule3d::new(0.1, 0.5)
						.mesh()
						.rings(1)
						.latitudes(8)
						.longitudes(16)
						.uv_profile(CapsuleUvProfile::Fixed),
				),
				material: gridbox_material("red", materials, asset_server),
				..default()
			},
			Sword::new(0.25, sword_pivot, EntityHashSet::from_iter(vec![body]), 0.8),
		))
		.set_parent(sword_pivot)
		.id();

	(sword_pivot, sword_blade)
}

pub fn animate_sword(
	mut commands: Commands,
	mut sword_blades: Query<(Entity, &mut Sword, &GlobalTransform, Option<&DamageSweep>)>,
	mut sword_pivots: Query<(Entity, &mut Transform, &mut InAnimation), With<SwordPivot>>,
	time: Res<Time>,
	fray: Query<&FrayMusic>,
	mut ev_hit: EventWriter<EntityDamaged>,
	asset_server: Res<AssetServer>,
) {
	let fray = fray.get_single().expect("Could not find fray");
	for (sword_blade_entity, mut sword_blade, sword_blade_global_transform, dealer) in
		sword_blades.iter_mut()
	{
		let (sword_pivot_entity, mut transform, mut animation) =
			ok_or_continue!(sword_pivots.get_mut(sword_blade.pivot));

		let prev_time = fray.time_to_bpm_beat(animation.time);
		animation.time += time.delta();
		let curr_time = fray.time_to_bpm_beat(animation.time);

		let follow_through_time = sword_blade.follow_through_time;

		if (prev_time..curr_time).contains(&0.0) {
			sword_blade.start_slash_time = fray.time;

			commands.entity(sword_blade_entity).insert(DamageSweep::new(
				*sword_blade_global_transform,
				sword_pivot_entity,
				sword_blade.allies.clone(),
			));

			commands.spawn((
				Name::new("Sword Swing SFX"),
				AudioBundle {
					source: asset_server.load("woosh.mp3"),
					settings: PlaybackSettings::DESPAWN,
				},
			));
		}
		if (prev_time..curr_time).contains(&follow_through_time) {
			commands.entity(sword_blade_entity).insert(EndDamageSweep);

			if let Some(dealer) = dealer {
				for entity in dealer.hit_entities.iter() {
					let mut fray = fray.clone();
					fray.time = sword_blade.start_slash_time;

					let damage = fray.modify_fray_damage(sword_blade.damage);
					let fray_modifier = fray.modify_fray_damage(1.0);
					ev_hit.send(EntityDamaged {
						victim: *entity,
						damage,
						fray_modifier,
					});
				}
			}

			sword_blade.side = sword_blade.side.other_side();
			commands.entity(sword_pivot_entity).remove::<InAnimation>();
		}

		let angle = if (0.0..follow_through_time).contains(&curr_time) {
			curr_time.map_range_ease(
				0.0..follow_through_time,
				sword_blade.side.angle()..sword_blade.side.other_side().angle(),
				EaseFunction::QuarticOut,
			)
		} else {
			sword_blade.side.angle()
		};
		transform.rotation = Quat::from_rotation_y(angle);
	}
}
