use std::f32::consts::{PI, TAU};
use std::time::Duration;

use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;
use bevy_rapier3d::math::Real;
use bevy_rapier3d::plugin::RapierContext;
use bevy_rapier3d::prelude::QueryFilter;
use interpolation::EaseFunction;

use crate::fray::FrayMusic;
use crate::gridbox_material;
use crate::player_controller::PlayerCamera;
use crate::util::MapRange;

use super::{DamageEvent, DamageSweep, EndDamageSweep, InAnimation, SweepPivot};

#[derive(Component)]
pub struct RiflePivot;

#[derive(Component)]
pub struct Rifle {
	pub damage: f32,
	pub pivot: Entity,
	pub allies: EntityHashSet,
	pub charge: f32,
	pub charge_rate: f32,
	pub max_charge: f32,
	pub full_charge_multiplier: f32,
}

pub fn spawn_rifle(
	commands: &mut Commands,
	asset_server: &AssetServer,
	materials: &mut Assets<StandardMaterial>,
	meshes: &mut Assets<Mesh>,
	body: Entity,
) -> (Entity, Entity) {
	let rifle_pivot = commands
		.spawn((
			Name::new("Rifle Pivot"),
			TransformBundle::from_transform(Transform::from_translation(Vec3::new(
				0.25, 0.0, -0.5,
			))),
			VisibilityBundle::default(),
			RiflePivot,
		))
		.set_parent(body)
		.id();

	let rifle_barrel = commands
		.spawn((
			Name::new("Rifle Barrel"),
			PbrBundle {
				transform: Transform::from_rotation(Quat::from_rotation_x(-PI / 2.)),
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
			Rifle {
				damage: 0.5,
				pivot: rifle_pivot,
				allies: EntityHashSet::from_iter(vec![body]),
				charge: 0.0,
				charge_rate: 1.0,
				max_charge: 1.0,
				full_charge_multiplier: 2.0,
			},
		))
		.set_parent(rifle_pivot)
		.id();

	(rifle_pivot, rifle_barrel)
}

pub fn animate_rifle(
	mut commands: Commands,
	mut rifle_barrels: Query<(Entity, &mut Rifle, &GlobalTransform)>,
	mut rifle_pivots: Query<(Entity, &mut Transform, &mut InAnimation), With<RiflePivot>>,
	time: Res<Time>,
	fray: Query<&FrayMusic>,
	mut ev_hit: EventWriter<DamageEvent>,
	asset_server: Res<AssetServer>,
	rapier_context: Res<RapierContext>,
	player_camera: Query<&GlobalTransform, With<PlayerCamera>>,
) {
	let fray = fray.get_single().expect("Could not find fray");
	for (rifle_barrel_entity, mut rifle_barrel, rifle_barrel_global_transform) in
		rifle_barrels.iter_mut()
	{
		let Ok((sword_pivot_entity, mut transform, mut animation)) =
			rifle_pivots.get_mut(rifle_barrel.pivot)
		else {
			continue;
		};
		let prev_time = fray.time_to_bpm_beat(animation.time);
		animation.time += time.delta();
		let curr_time = fray.time_to_bpm_beat(animation.time);

		if (prev_time..curr_time).contains(&0.0) {
			commands.spawn((
				Name::new("Rifle Shot SFX"),
				AudioBundle {
					source: asset_server.load("flute.wav"),
					settings: PlaybackSettings::DESPAWN,
				},
			));

			let player_camera = player_camera.get_single().expect("Player camera not found");
			if let Some((hit_entity, _distance)) = rapier_context.cast_ray(
				player_camera.translation(),
				player_camera.forward().into(),
				Real::MAX,
				false,
				QueryFilter::new().predicate(&|entity| !rifle_barrel.allies.contains(&entity)),
			) {
				let damage = fray.modify_fray_damage(rifle_barrel.damage);
				let fray_modifier = fray.modify_fray_damage(1.0);
				ev_hit.send(DamageEvent {
					victim: hit_entity,
					damage,
					fray_modifier,
				});
			}
		}
		if (prev_time..curr_time).contains(&1.5) {
			commands.entity(sword_pivot_entity).remove::<InAnimation>();
		}

		let angle = match curr_time {
			0.0..1.5 => curr_time.map_range_ease(0.0..1.5, 0.0..TAU, EaseFunction::QuarticOut),
			_ => 0.0,
		};
		transform.rotation = Quat::from_rotation_x(angle);
	}
}
