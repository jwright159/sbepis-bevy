use bevy::prelude::*;

#[derive(Component)]
pub struct Health(pub f32);

#[derive(Component)]
pub struct CanDealDamage;

#[derive(Component)]
pub struct SpawnHealthBar;

#[derive(Component)]
pub struct HealthBar {
	pub entity: Entity,
	pub health: f32,
	pub max_health: f32,
}

pub fn kill_entities_with_no_health(mut commands: Commands, healths: Query<(Entity, &Health)>) {
	for (entity, health) in healths.iter() {
		if health.0 <= 0. {
			commands.entity(entity).despawn_recursive();
		}
	}
}
