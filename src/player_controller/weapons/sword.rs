use std::f32::consts::PI;
use std::time::Duration;

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
	pub start_slash_time: Duration,
	side: SwordSide,
}

impl Sword {
	pub fn new(damage: f32, pivot: Entity) -> Self {
		Self {
			damage,
			pivot,
			start_slash_time: Duration::ZERO,
			side: SwordSide::Left,
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

pub fn animate_sword(
	mut commands: Commands,
	mut sword_blades: Query<(Entity, &mut Sword, Option<&CanDealDamage>)>,
	mut sword_pivots: Query<(Entity, &mut Transform, &mut InAnimation), With<SwordPivot>>,
	time: Res<Time>,
	fray: Query<&FrayMusic>,
	mut ev_hit: EventWriter<DamageEvent>,
	asset_server: Res<AssetServer>,
) {
	let fray = fray.get_single().expect("Could not find fray");
	for (sword_blade_entity, mut sword_blade, dealer) in sword_blades.iter_mut() {
		let Ok((sword_pivot_entity, mut transform, mut animation)) =
			sword_pivots.get_mut(sword_blade.pivot)
		else {
			continue;
		};
		let prev_time = fray.time_to_bpm_beat(animation.time);
		animation.time += time.delta();
		let curr_time = fray.time_to_bpm_beat(animation.time);

		if (prev_time..curr_time).contains(&0.0) {
			sword_blade.start_slash_time = fray.time;

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
		if (prev_time..curr_time).contains(&0.6) {
			commands
				.entity(sword_blade_entity)
				.remove::<CanDealDamage>();

			if let Some(dealer) = dealer {
				for entity in dealer.hit_entities.iter() {
					let mut fray = fray.clone();
					fray.time = sword_blade.start_slash_time;

					let damage = fray.modify_fray_damage(sword_blade.damage);
					let fray_modifier = fray.modify_fray_damage(1.0);
					ev_hit.send(DamageEvent {
						victim: *entity,
						damage,
						fray_modifier,
					});
				}
			}

			sword_blade.side = sword_blade.side.other_side();
			commands.entity(sword_pivot_entity).remove::<InAnimation>();
		}

		let angle = match curr_time {
			0.0..0.6 => curr_time
				.map_range(0.0..0.6, 0.0..(PI * 0.5))
				.cos()
				.map_range(
					0.0..1.0,
					sword_blade.side.other_side().angle()..sword_blade.side.angle(),
				),
			_ => sword_blade.side.angle(),
		};
		transform.rotation = Quat::from_rotation_y(angle);
	}
}
