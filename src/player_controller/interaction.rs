use std::f32::consts::PI;
use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier3d::pipeline::CollisionEvent;

use crate::entity::health::CanDealDamage;
use crate::entity::GelViscosity;
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
	mut hammer_pivots: Query<(Entity, &mut Transform, &mut InAnimation), With<HammerPivot>>,
	hammer_heads: Query<(Entity, &Hammer)>,
	time: Res<Time>,
) {
	for (hammer_pivot, mut transform, mut animation) in hammer_pivots.iter_mut() {
		let (hammer_head, _) = hammer_heads
			.iter()
			.find(|(_, head)| head.pivot == hammer_pivot)
			.expect("Hammer pivot found without hammer head");
		animation.time += time.delta();
		let time = animation.time.as_secs_f32();
		let angle = match time {
			0.0..0.2 => {
				commands.entity(hammer_head).insert(CanDealDamage);
				time.map_range(0.0..0.2, 0.0..(PI * 0.5))
					.cos()
					.map_range(0.0..1.0, (-PI * 0.5)..0.0)
			}
			0.2..0.5 => {
				commands.entity(hammer_head).remove::<CanDealDamage>();
				time.map_range(0.2..0.5, 0.0..PI)
					.cos()
					.map_range(-1.0..1.0, 0.0..(-PI * 0.5))
			}
			_ => {
				commands.entity(hammer_pivot).remove::<InAnimation>();
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
) {
	for event in collision_events.read() {
		if let CollisionEvent::Started(a, b, _flags) = event {
			if let (Ok(hammer), Ok((entity, health))) = (hammers.get(*a), healths.get_mut(*b)) {
				damage_with_hammer(&mut commands, entity, health, hammer);
			}
			if let (Ok(hammer), Ok((entity, health))) = (hammers.get(*b), healths.get_mut(*a)) {
				damage_with_hammer(&mut commands, entity, health, hammer);
			}
		}
	}
}

fn damage_with_hammer(
	commands: &mut Commands,
	entity: Entity,
	mut health: Mut<GelViscosity>,
	hammer: &Hammer,
) {
	if health.value <= 0.0 {
		commands.entity(entity).despawn_recursive();
		return;
	}

	health.value -= hammer.damage;
}
