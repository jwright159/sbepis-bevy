use bevy::prelude::*;

#[derive(Component)]
pub struct Health(pub f32);

#[derive(Component)]
pub struct CanDealDamage;

pub fn kill_entities_with_no_health(mut commands: Commands, healths: Query<(Entity, &Health)>) {
	for (entity, health) in healths.iter() {
		if health.0 <= 0. {
			commands.entity(entity).despawn_recursive();
		}
	}
}
