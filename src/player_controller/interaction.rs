use std::f32::consts::PI;
use bevy::prelude::*;
use bevy_rapier3d::pipeline::CollisionEvent;

#[derive(Component)]
pub struct HammerPivot;

#[derive(Component)]
pub struct Hammer
{
	pub damage: f32,
}

#[derive(Component)]
pub struct Health
{
	pub health: f32,
}

pub fn attack(
	In(attacking): In<bool>,
	mut hammer_pivot: Query<&mut Transform, With<HammerPivot>>,
) {
	let mut hammer_pivot = hammer_pivot.single_mut();
	hammer_pivot.rotation = if attacking { Quat::from_rotation_x(-PI / 2.) } else { Quat::IDENTITY };
}

pub fn collide_hammer(
	mut collision_events: EventReader<CollisionEvent>,
	hammers: Query<&Hammer>,
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

fn damage_with_hammer(
	mut health: Mut<Health>,
	hammer: &Hammer,
) {
	health.health -= hammer.damage;
	info!("Hammer dealt {} damage, health is now {}", hammer.damage, health.health);
}

pub fn kill_entities_with_no_health(
	mut commands: Commands,
	healths: Query<(Entity, &Health)>,
) {
	for (entity, health) in healths.iter() {
		if health.health <= 0. {
			commands.entity(entity).despawn_recursive();
		}
	}
}