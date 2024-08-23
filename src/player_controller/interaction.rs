use std::f32::consts::PI;
use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier3d::pipeline::CollisionEvent;

use crate::entity::health::CanDealDamage;
use crate::entity::GelViscosity;
use crate::fray::FrayMusic;
use crate::util::MapRange;

#[derive(Component)]
pub struct HammerPivot;

#[derive(Component)]
pub struct Hammer {
	pub damage: f32,
	pub pivot: Entity,
}

#[derive(Component, Default)]
pub struct InAnimation {
	pub time: Duration,
}

pub fn attack(
	mut commands: Commands,
	hammers: Query<Entity, (With<HammerPivot>, Without<InAnimation>)>,
) {
	for hammer in hammers.iter() {
		commands.entity(hammer).insert(InAnimation::default());
	}
}

pub fn animate_hammer(
	mut commands: Commands,
	hammer_heads: Query<(Entity, &Hammer)>,
	mut hammer_pivots: Query<(Entity, &mut Transform, &mut InAnimation), With<HammerPivot>>,
	time: Res<Time>,
	fray: Query<&FrayMusic>,
) {
	let fray = fray.get_single().expect("Could not find fray");
	for (hammer_head_entity, hammer_head) in hammer_heads.iter() {
		let Ok((hammer_pivot_entity, mut transform, mut animation)) =
			hammer_pivots.get_mut(hammer_head.pivot)
		else {
			continue;
		};
		animation.time += time.delta();
		let time = fray.time_to_bpm_beat(animation.time);
		let angle = match time {
			0.0..0.5 => {
				commands.entity(hammer_head_entity).insert(CanDealDamage);
				time.map_range(0.0..0.5, 0.0..(PI * 0.5))
					.cos()
					.map_range(0.0..1.0, (-PI * 0.5)..0.0)
			}
			0.5..3.5 => {
				commands
					.entity(hammer_head_entity)
					.remove::<CanDealDamage>();
				time.map_range(0.5..3.5, 0.0..PI)
					.cos()
					.map_range(-1.0..1.0, 0.0..(-PI * 0.5))
			}
			_ => {
				commands.entity(hammer_pivot_entity).remove::<InAnimation>();
				0.0
			}
		};
		transform.rotation = Quat::from_rotation_x(angle);
	}
}

pub fn collide_hammer(
	mut commands: Commands,
	mut collision_events: EventReader<CollisionEvent>,
	hammers: Query<&Hammer, With<CanDealDamage>>,
	mut healths: Query<(Entity, &mut GelViscosity)>,
	frays: Query<&FrayMusic>,
) {
	let fray = frays.get_single().expect("No fray music found");
	for event in collision_events.read() {
		if let CollisionEvent::Started(a, b, _flags) = event {
			if let (Ok(hammer), Ok((entity, health))) = (hammers.get(*a), healths.get_mut(*b)) {
				damage_with_hammer(&mut commands, entity, health, hammer, fray);
			}
			if let (Ok(hammer), Ok((entity, health))) = (hammers.get(*b), healths.get_mut(*a)) {
				damage_with_hammer(&mut commands, entity, health, hammer, fray);
			}
		}
	}
}

fn damage_with_hammer(
	commands: &mut Commands,
	entity: Entity,
	mut health: Mut<GelViscosity>,
	hammer: &Hammer,
	fray: &FrayMusic,
) {
	let damage = fray.modify_fray_damage(hammer.damage);

	if damage > 0.0 && health.value <= 0.0 {
		commands.entity(entity).despawn_recursive();
		return;
	}

	health.value -= damage;
}
