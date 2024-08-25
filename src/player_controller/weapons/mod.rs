use std::time::Duration;

use bevy::prelude::*;
use bevy_rapier3d::pipeline::CollisionEvent;

use crate::entity::health::CanDealDamage;
use crate::entity::GelViscosity;

pub mod hammer;

#[derive(Component, Default)]
pub struct InAnimation {
	pub time: Duration,
}

#[derive(Event)]
pub struct DamageEvent {
	pub victim: Entity,
	pub damage: f32,
}

#[derive(Component)]
pub struct DamageNumbers;

#[derive(Component)]
pub struct ActiveWeapon;

pub fn collide_dealers(
	mut ev_collision: EventReader<CollisionEvent>,
	mut dealers: Query<&mut CanDealDamage>,
	healths: Query<Entity, With<GelViscosity>>,
) {
	for event in ev_collision.read() {
		if let CollisionEvent::Started(a, b, _flags) = event {
			if let (Ok(mut dealer), Ok(entity)) = (dealers.get_mut(*a), healths.get(*b)) {
				dealer.hit_entities.push(entity);
			}
			if let (Ok(mut dealer), Ok(entity)) = (dealers.get_mut(*b), healths.get(*a)) {
				dealer.hit_entities.push(entity);
			}
		}
	}
}

pub fn deal_all_damage(
	mut ev_hit: EventReader<DamageEvent>,
	mut commands: Commands,
	mut healths: Query<&mut GelViscosity>,
) {
	for event in ev_hit.read() {
		let damage = event.damage;
		let mut health = healths.get_mut(event.victim).unwrap();

		if damage > 0.0 && health.value <= 0.0 {
			commands.entity(event.victim).despawn_recursive();
			return;
		}

		health.value -= damage;
	}
}
