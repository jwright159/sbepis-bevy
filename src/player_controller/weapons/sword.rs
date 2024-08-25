use std::f32::consts::PI;

use bevy::prelude::*;

use crate::entity::health::CanDealDamage;
use crate::fray::FrayMusic;
use crate::util::MapRange;

use super::{DamageEvent, InAnimation};

#[derive(Component)]
pub struct SwordPivot;

#[derive(Component)]
pub struct Sword {
	pub damage: f32,
	pub pivot: Entity,
}

pub fn animate_sword(
	mut commands: Commands,
	sword_blades: Query<(Entity, &Sword, Option<&CanDealDamage>)>,
	mut sword_pivots: Query<(Entity, &mut Transform, &mut InAnimation), With<SwordPivot>>,
	time: Res<Time>,
	fray: Query<&FrayMusic>,
	mut ev_hit: EventWriter<DamageEvent>,
	asset_server: Res<AssetServer>,
) {
	let fray = fray.get_single().expect("Could not find fray");
	for (sword_blade_entity, sword_blade, dealer) in sword_blades.iter() {
		let Ok((sword_pivot_entity, mut transform, mut animation)) =
			sword_pivots.get_mut(sword_blade.pivot)
		else {
			continue;
		};
		let prev_time = fray.time_to_bpm_beat(animation.time);
		animation.time += time.delta();
		let time = fray.time_to_bpm_beat(animation.time);

		if (prev_time..time).contains(&0.0) {
			commands
				.entity(sword_blade_entity)
				.insert(CanDealDamage::default());

			commands.spawn((
				Name::new("Sword Swing SFX"),
				AudioBundle {
					source: asset_server.load("woosh.mp3"),
					settings: PlaybackSettings::DESPAWN,
				},
			));
		}
		if (prev_time..time).contains(&0.25) {
			commands
				.entity(sword_blade_entity)
				.remove::<CanDealDamage>();

			commands.spawn((
				Name::new("Sword Slash SFX"),
				AudioBundle {
					source: asset_server.load("concrete_break3.wav"),
					settings: PlaybackSettings::DESPAWN,
				},
			));

			if let Some(dealer) = dealer {
				for entity in dealer.hit_entities.iter() {
					let damage = fray.modify_fray_damage(sword_blade.damage);
					let fray_modifier = fray.modify_fray_damage(1.0);
					ev_hit.send(DamageEvent {
						victim: *entity,
						damage,
						fray_modifier,
					});
				}
			}
		}
		if (prev_time..time).contains(&0.75) {
			commands.entity(sword_pivot_entity).remove::<InAnimation>();
		}

		let angle = match time {
			0.0..0.25 => time
				.map_range(0.0..0.25, 0.0..(PI * 0.5))
				.cos()
				.map_range(0.0..1.0, (PI * 0.5)..(-PI * 0.5)),
			0.25..0.75 => time
				.map_range(0.25..0.75, 0.0..PI)
				.cos()
				.map_range(-1.0..1.0, (-PI * 0.5)..(PI * 0.5)),
			_ => -PI * 0.5,
		};
		transform.rotation = Quat::from_rotation_y(angle);
	}
}
