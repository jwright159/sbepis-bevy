use crate::util::MapRange;
use bevy::prelude::*;
use bevy_rapier3d::pipeline::CollisionEvent;
use std::f32::consts::PI;
use std::time::Duration;

#[derive(Component)]
pub struct HammerPivot;

#[derive(Component)]
pub struct Hammer {
	pub damage: f32,
	pub pivot: Entity,
}

#[derive(Component)]
pub struct Health {
	pub health: f32,
}

#[derive(Component, Default)]
pub struct InAnimation {
	pub time: Duration,
}

#[derive(Component)]
pub struct CanDealDamage;

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
	mut collision_events: EventReader<CollisionEvent>,
	hammers: Query<&Hammer, With<CanDealDamage>>,
	mut healths: Query<&mut Health>,
) {
	for event in collision_events.read() {
		if let CollisionEvent::Started(a, b, _flags) = event {
			if let (Ok(hammer), Ok(health)) = (hammers.get(*a), healths.get_mut(*b)) {
				damage_with_hammer(health, hammer);
			}
			if let (Ok(hammer), Ok(health)) = (hammers.get(*b), healths.get_mut(*a)) {
				damage_with_hammer(health, hammer);
			}
		}
	}
}

fn damage_with_hammer(mut health: Mut<Health>, hammer: &Hammer) {
	health.health -= hammer.damage;
	info!(
		"Hammer dealt {} damage, health is now {}",
		hammer.damage, health.health
	);
}

pub fn kill_entities_with_no_health(mut commands: Commands, healths: Query<(Entity, &Health)>) {
	for (entity, health) in healths.iter() {
		if health.health <= 0. {
			commands.entity(entity).despawn_recursive();
		}
	}
}
