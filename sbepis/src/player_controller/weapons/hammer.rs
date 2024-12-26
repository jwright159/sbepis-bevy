use std::f32::consts::PI;

use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;
use interpolation::EaseFunction;

use crate::fray::FrayMusic;
use crate::util::MapRange;
use crate::{gridbox_material, ok_or_continue};

use super::{DamageSweep, EndDamageSweep, EntityDamaged, InAnimation, SweepPivot};

#[derive(Component)]
pub struct HammerPivot;

#[derive(Component)]
pub struct Hammer {
	pub damage: f32,
	pub pivot: Entity,
	pub allies: EntityHashSet,
	pub lead_in_time: f32,
	pub follow_through_time: f32,
}

pub fn spawn_hammer(
	commands: &mut Commands,
	asset_server: &AssetServer,
	materials: &mut Assets<StandardMaterial>,
	meshes: &mut Assets<Mesh>,
	body: Entity,
) -> (Entity, Entity) {
	let hammer_pivot = commands
		.spawn((
			Name::new("Hammer Pivot"),
			HammerPivot,
			SweepPivot {
				sweeper_length: 0.2,
				sweep_depth: 0.5,
				sweep_height: 0.2,
			},
		))
		.set_parent(body)
		.id();

	let hammer_head = commands
		.spawn((
			Name::new("Hammer Head"),
			Transform::default()
				.with_translation(Vec3::Y * 1.)
				.with_rotation(Quat::from_rotation_x(PI / 2.)),
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
			Hammer {
				damage: 1.0,
				pivot: hammer_pivot,
				allies: EntityHashSet::from_iter(vec![body]),
				lead_in_time: 0.5,
				follow_through_time: 3.0,
			},
		))
		.set_parent(hammer_pivot)
		.id();

	(hammer_pivot, hammer_head)
}

pub fn animate_hammer(
	mut commands: Commands,
	mut hammer_heads: Query<(Entity, &Hammer, &GlobalTransform, Option<&mut DamageSweep>)>,
	mut hammer_pivots: Query<(Entity, &mut Transform, &mut InAnimation), With<HammerPivot>>,
	time: Res<Time>,
	fray: Query<&FrayMusic>,
	mut ev_hit: EventWriter<EntityDamaged>,
	asset_server: Res<AssetServer>,
) {
	let fray = fray.get_single().expect("Could not find fray");
	for (hammer_head_entity, hammer_head, hammer_head_global_transform, dealer) in
		hammer_heads.iter_mut()
	{
		let (hammer_pivot_entity, mut transform, mut animation) =
			ok_or_continue!(hammer_pivots.get_mut(hammer_head.pivot));

		let prev_time = fray.time_to_bpm_beat(animation.time) as f32;
		animation.time += time.delta();
		let curr_time = fray.time_to_bpm_beat(animation.time) as f32;

		let lead_in_time = hammer_head.lead_in_time;
		let follow_through_time = lead_in_time + hammer_head.follow_through_time;

		if (prev_time..curr_time).contains(&0.0) {
			commands.entity(hammer_head_entity).insert(DamageSweep::new(
				*hammer_head_global_transform,
				hammer_pivot_entity,
				hammer_head.allies.clone(),
			));

			commands.spawn((
				Name::new("Hammer Swing SFX"),
				AudioPlayer::new(asset_server.load("whoosh.mp3")),
				PlaybackSettings::DESPAWN,
			));
		}
		if (prev_time..curr_time).contains(&lead_in_time) {
			commands.entity(hammer_head_entity).insert(EndDamageSweep);

			commands.spawn((
				Name::new("Hammer Smash SFX"),
				AudioPlayer::new(asset_server.load("concrete_break3.wav")),
				PlaybackSettings::DESPAWN,
			));

			if let Some(dealer) = dealer {
				for entity in dealer.hit_entities.iter() {
					let damage = fray.modify_fray_damage(hammer_head.damage);
					let fray_modifier = fray.modify_fray_damage(1.0);
					ev_hit.send(EntityDamaged {
						victim: *entity,
						damage,
						fray_modifier,
					});
				}
			}
		}
		if (prev_time..curr_time).contains(&follow_through_time) {
			commands.entity(hammer_pivot_entity).remove::<InAnimation>();
		}

		let angle = if (0.0..lead_in_time).contains(&curr_time) {
			curr_time.map_range_ease(
				0.0..lead_in_time,
				0.0..(-PI * 0.5),
				EaseFunction::ExponentialIn,
			)
		} else if (lead_in_time..follow_through_time).contains(&curr_time) {
			curr_time.map_range_ease(
				lead_in_time..follow_through_time,
				(-PI * 0.5)..0.0,
				EaseFunction::CubicInOut,
			)
		} else {
			0.0
		};

		transform.rotation = Quat::from_rotation_x(angle);
	}
}
