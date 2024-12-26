use std::f32::consts::{PI, TAU};

use avian3d::prelude::*;
use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;
use interpolation::EaseFunction;

use crate::camera::PlayerCamera;
use crate::fray::FrayMusic;
use crate::util::MapRange;
use crate::{gridbox_material, ok_or_continue};

use super::{EntityDamaged, InAnimation};

#[derive(Component)]
pub struct RiflePivot;

#[derive(Component)]
pub struct Rifle {
	pub damage: f32,
	pub pivot: Entity,
	pub allies: EntityHashSet,
	pub charge: u32,
	pub last_beat: u32,
	pub charge_rate: u32,
	pub max_charge: u32,
	pub full_charge_multiplier: f32,
	pub reload_time: f32,
}

impl Rifle {
	fn update_last_beat(&mut self, fray: &FrayMusic) {
		self.last_beat = fray.subbeats(self.charge_rate);
	}

	fn get_beat(&mut self, fray: &FrayMusic) -> u32 {
		fray.subbeats(self.charge_rate)
	}
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
			Transform::from_translation(Vec3::new(0.25, 0.0, -0.5)),
			Visibility::default(),
			RiflePivot,
		))
		.set_parent(body)
		.id();

	let rifle_barrel = commands
		.spawn((
			Name::new("Rifle Barrel"),
			Transform::from_rotation(Quat::from_rotation_x(-PI / 2.)),
			Mesh3d(
				meshes.add(
					Capsule3d::new(0.1, 0.5)
						.mesh()
						.rings(1)
						.latitudes(8)
						.longitudes(16)
						.uv_profile(CapsuleUvProfile::Fixed),
				),
			),
			MeshMaterial3d(gridbox_material("red", materials, asset_server)),
			Rifle {
				damage: 0.5,
				pivot: rifle_pivot,
				allies: EntityHashSet::from_iter(vec![body]),
				charge: 0,
				last_beat: 0,
				charge_rate: 2,
				max_charge: 4,
				full_charge_multiplier: 3.0,
				reload_time: 1.75,
			},
		))
		.set_parent(rifle_pivot)
		.id();

	(rifle_pivot, rifle_barrel)
}

pub fn animate_rifle(
	mut commands: Commands,
	mut rifle_barrels: Query<&mut Rifle>,
	mut rifle_pivots: Query<(Entity, &mut Transform, &mut InAnimation), With<RiflePivot>>,
	time: Res<Time>,
	fray: Query<&FrayMusic>,
	mut ev_hit: EventWriter<EntityDamaged>,
	asset_server: Res<AssetServer>,
	ray_hits: Query<&RayHits, With<PlayerCamera>>,
) {
	let fray = fray.get_single().expect("Could not find fray");
	for mut rifle_barrel in rifle_barrels.iter_mut() {
		let (rifle_barrel_entity, mut transform, mut animation) =
			ok_or_continue!(rifle_pivots.get_mut(rifle_barrel.pivot));

		let prev_time = fray.time_to_bpm_beat(animation.time) as f32;
		animation.time += time.delta();
		let curr_time = fray.time_to_bpm_beat(animation.time) as f32;

		let reload_time = rifle_barrel.reload_time;

		if (prev_time..curr_time).contains(&0.0) {
			commands.spawn((
				Name::new("Rifle Shot SFX"),
				AudioPlayer::new(asset_server.load("flute.wav")),
				PlaybackSettings::DESPAWN,
			));

			let ray_hits = ray_hits.get_single().expect("Ray hits not found");
			if let Some(RayHitData {
				entity: hit_entity, ..
			}) = ray_hits.iter().next()
			{
				if !rifle_barrel.allies.contains(hit_entity) {
					let charge_multiplier = if rifle_barrel.charge >= rifle_barrel.max_charge {
						rifle_barrel.full_charge_multiplier
					} else {
						1.0
					};
					rifle_barrel.charge = 0;
					let damage = fray.modify_fray_damage(rifle_barrel.damage) * charge_multiplier;
					let fray_modifier = fray.modify_fray_damage(1.0);
					ev_hit.send(EntityDamaged {
						victim: *hit_entity,
						damage,
						fray_modifier,
					});
				}
			}
		}
		if (prev_time..curr_time).contains(&reload_time) {
			commands.entity(rifle_barrel_entity).remove::<InAnimation>();
			rifle_barrel.update_last_beat(fray);
		}

		let angle = if (0.0..reload_time).contains(&curr_time) {
			curr_time.map_range_ease(0.0..reload_time, 0.0..TAU, EaseFunction::QuarticOut)
		} else {
			0.0
		};
		transform.rotation = Quat::from_rotation_x(angle);
	}
}

pub fn charge_rifle(
	mut commands: Commands,
	mut rifle_barrels: Query<&mut Rifle>,
	rifle_pivots: Query<Entity, (With<RiflePivot>, Without<InAnimation>)>,
	fray: Query<&FrayMusic>,
	asset_server: Res<AssetServer>,
) {
	let fray = fray.get_single().expect("Could not find fray");
	for mut rifle_barrel in rifle_barrels.iter_mut() {
		if rifle_pivots.get(rifle_barrel.pivot).is_err() {
			continue;
		};

		let beat = rifle_barrel.get_beat(fray);
		if rifle_barrel.charge < rifle_barrel.max_charge && rifle_barrel.last_beat != beat {
			rifle_barrel.charge += 1;

			commands.spawn((
				Name::new("Rifle Charge SFX"),
				AudioPlayer::new(asset_server.load("flute.wav")),
				PlaybackSettings::DESPAWN.with_speed(2.0),
			));
		}
		rifle_barrel.update_last_beat(fray);
	}
}
