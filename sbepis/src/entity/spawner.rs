use std::time::Duration;

use bevy::prelude::*;

#[derive(Component)]
pub struct Spawner {
	pub max_amount: usize,
	pub spawn_delay: Duration,
	pub spawn_timer: Duration,
}

#[derive(Component)]
pub struct SpawnedEntity {
	pub spawner: Entity,
}

pub struct SpawnEntityInformation {
	pub spawner: Entity,
	pub position: Vec3,
}

pub fn spawn_entities<SpawnerType: Component, EntityType: Component>(
	mut spawners: Query<(Entity, &mut Spawner, &GlobalTransform), With<SpawnerType>>,
	entities: Query<&SpawnedEntity, With<EntityType>>,
	time: Res<Time>,
) -> Option<SpawnEntityInformation> {
	let mut spawn_info = None;
	for (spawner_entity, mut spawner, transform) in spawners.iter_mut() {
		spawner.spawn_timer += time.delta();

		let entity_count = entities
			.iter()
			.filter(|e| e.spawner == spawner_entity)
			.count();
		if spawner.spawn_timer >= spawner.spawn_delay && entity_count < spawner.max_amount {
			spawner.spawn_timer = Duration::ZERO;
			spawn_info = Some(SpawnEntityInformation {
				spawner: spawner_entity,
				position: transform.translation(),
			});
		}
	}
	spawn_info
}
