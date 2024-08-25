use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;
use interpolation::EaseFunction;

use crate::fray::FrayMusic;
use crate::gridbox_material;
use crate::util::MapRange;

use super::{DamageEvent, DamageSweep, EndDamageSweep, InAnimation, SweepPivot};

#[derive(Component)]
pub struct HammerPivot;

#[derive(Component)]
pub struct Hammer {
	pub damage: f32,
	pub pivot: Entity,
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
			TransformBundle::from_transform(Transform::from_translation(Vec3::ZERO)),
			VisibilityBundle::default(),
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
			PbrBundle {
				transform: Transform::default()
					.with_translation(Vec3::Y * 1.)
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
			Hammer {
				damage: 1.0,
				pivot: hammer_pivot,
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
	mut ev_hit: EventWriter<DamageEvent>,
	asset_server: Res<AssetServer>,
) {
	let fray = fray.get_single().expect("Could not find fray");
	for (hammer_head_entity, hammer_head, hammer_head_global_transform, dealer) in
		hammer_heads.iter_mut()
	{
		let Ok((hammer_pivot_entity, mut transform, mut animation)) =
			hammer_pivots.get_mut(hammer_head.pivot)
		else {
			continue;
		};
		let prev_time = fray.time_to_bpm_beat(animation.time);
		animation.time += time.delta();
		let time = fray.time_to_bpm_beat(animation.time);

		if (prev_time..time).contains(&0.0) {
			commands.entity(hammer_head_entity).insert(DamageSweep::new(
				*hammer_head_global_transform,
				hammer_pivot_entity,
			));

			commands.spawn((
				Name::new("Hammer Swing SFX"),
				AudioBundle {
					source: asset_server.load("woosh.mp3"),
					settings: PlaybackSettings::DESPAWN,
				},
			));
		}
		if (prev_time..time).contains(&0.5) {
			commands.entity(hammer_head_entity).insert(EndDamageSweep);

			commands.spawn((
				Name::new("Hammer Smash SFX"),
				AudioBundle {
					source: asset_server.load("concrete_break3.wav"),
					settings: PlaybackSettings::DESPAWN,
				},
			));

			if let Some(dealer) = dealer {
				for entity in dealer.hit_entities.iter() {
					let damage = fray.modify_fray_damage(hammer_head.damage);
					let fray_modifier = fray.modify_fray_damage(1.0);
					ev_hit.send(DamageEvent {
						victim: *entity,
						damage,
						fray_modifier,
					});
				}
			}
		}
		if (prev_time..time).contains(&3.5) {
			commands.entity(hammer_pivot_entity).remove::<InAnimation>();
		}

		let angle = match time {
			0.0..0.5 => {
				time.map_range_ease(0.0..0.5, 0.0..(-PI * 0.5), EaseFunction::ExponentialIn)
			}
			0.5..3.5 => time.map_range_ease(0.5..3.5, (-PI * 0.5)..0.0, EaseFunction::CubicInOut),
			_ => 0.0,
		};
		transform.rotation = Quat::from_rotation_x(angle);
	}
}
