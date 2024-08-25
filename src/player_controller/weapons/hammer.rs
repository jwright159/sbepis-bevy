use std::f32::consts::PI;

use bevy::prelude::*;

use crate::entity::health::CanDealDamage;
use crate::fray::FrayMusic;
use crate::util::MapRange;

use super::{DamageEvent, InAnimation};

#[derive(Component)]
pub struct HammerPivot;

#[derive(Component)]
pub struct Hammer {
	pub damage: f32,
	pub pivot: Entity,
}

pub fn animate_hammer(
	mut commands: Commands,
	hammer_heads: Query<(Entity, &Hammer, Option<&CanDealDamage>)>,
	mut hammer_pivots: Query<(Entity, &mut Transform, &mut InAnimation), With<HammerPivot>>,
	time: Res<Time>,
	fray: Query<&FrayMusic>,
	mut ev_hit: EventWriter<DamageEvent>,
	asset_server: Res<AssetServer>,
) {
	let fray = fray.get_single().expect("Could not find fray");
	for (hammer_head_entity, hammer_head, dealer) in hammer_heads.iter() {
		let Ok((hammer_pivot_entity, mut transform, mut animation)) =
			hammer_pivots.get_mut(hammer_head.pivot)
		else {
			continue;
		};
		let prev_time = fray.time_to_bpm_beat(animation.time);
		animation.time += time.delta();
		let time = fray.time_to_bpm_beat(animation.time);

		if (prev_time..time).contains(&0.0) {
			commands
				.entity(hammer_head_entity)
				.insert(CanDealDamage::default());

			commands.spawn((
				Name::new("Hammer Swing SFX"),
				AudioBundle {
					source: asset_server.load("woosh.mp3"),
					settings: PlaybackSettings::DESPAWN,
				},
			));
		}
		if (prev_time..time).contains(&0.5) {
			commands
				.entity(hammer_head_entity)
				.remove::<CanDealDamage>();

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
			0.0..0.5 => time
				.map_range(0.0..0.5, 0.0..(PI * 0.5))
				.cos()
				.map_range(0.0..1.0, (-PI * 0.5)..0.0),
			0.5..3.5 => time
				.map_range(0.5..3.5, 0.0..PI)
				.cos()
				.map_range(-1.0..1.0, 0.0..(-PI * 0.5)),
			_ => 0.0,
		};
		transform.rotation = Quat::from_rotation_x(angle);
	}
}
