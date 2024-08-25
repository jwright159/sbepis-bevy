use std::time::Duration;

use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_rapier3d::pipeline::CollisionEvent;

use crate::entity::health::CanDealDamage;
use crate::entity::GelViscosity;

pub mod hammer;
pub mod sword;

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

pub fn attack(
	mut commands: Commands,
	swords: Query<Entity, (Without<InAnimation>, With<ActiveWeapon>)>,
) {
	for hammer in swords.iter() {
		commands.entity(hammer).insert(InAnimation::default());
	}
}

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

pub fn update_damage_numbers(
	mut ev_hit: EventReader<DamageEvent>,
	mut damage_numbers: Query<&mut Text, With<DamageNumbers>>,
) {
	for event in ev_hit.read() {
		let damage = event.damage;
		for mut damage_numbers in damage_numbers.iter_mut() {
			damage_numbers.sections.push(TextSection::new(
				format!("\n{damage:.2}"),
				TextStyle {
					color: Color::mix(
						&Color::from(css::RED),
						&Color::from(css::GREEN),
						damage.clamp(0.0, 1.0),
					),
					..default()
				},
			));
		}
	}
}
