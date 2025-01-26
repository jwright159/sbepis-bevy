use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;
use bevy_butler::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_rapier3d::geometry::Collider;
use name_tags::*;

use crate::entity::spawner::{
	EntitySpawned, EntitySpawnedSet, SpawnerActivated, SpawnerActivatedSet,
};
use crate::entity::{Healing, RandomInput, RotateTowardMovement, SpawnHealthBar, TargetPlayer};
use crate::gridbox_material;
use crate::main_bundles::EntityBundle;
use crate::questing::{QuestGiver, SpawnQuestMarker};

mod name_tags;

#[butler_plugin(build(
	add_plugins(RonAssetPlugin::<AvailableNames>::new(&["names.ron"])),
	init_resource::<FontMeshGenerator>(),
))]
pub struct NpcPlugin;

#[derive(Component)]
pub struct Consort;

#[derive(Component)]
pub struct ConsortSpawner;

#[derive(Component)]
pub struct Imp;

#[derive(Component)]
pub struct ImpSpawner;

#[system(
	plugin = NpcPlugin, schedule = Update,
	after = SpawnerActivatedSet,
	in_set = EntitySpawnedSet,
)]
fn spawn_consort(
	mut ev_spawner: EventReader<SpawnerActivated>,
	mut ev_spawned: EventWriter<EntitySpawned>,
	spawners: Query<(), With<ConsortSpawner>>,
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
) {
	for ev in ev_spawner.read() {
		if spawners.get(ev.spawner).is_err() {
			continue;
		}

		commands.entity(ev.entity).insert((
			Name::new("Consort"),
			EntityBundle::new(
				Transform::from_translation(ev.position),
				meshes.add(
					Capsule3d::new(0.25, 0.5)
						.mesh()
						.rings(1)
						.latitudes(8)
						.longitudes(16)
						.uv_profile(CapsuleUvProfile::Fixed),
				),
				gridbox_material("magenta", &mut materials, &asset_server),
				Collider::capsule_y(0.25, 0.25),
			),
			SpawnHealthBar,
			RandomInput::default(),
			Healing(0.2),
			RotateTowardMovement,
			Consort,
			QuestGiver::default(),
			SpawnQuestMarker,
			SpawnNameTag,
		));
		ev_spawned.send(EntitySpawned(ev.entity));
	}
}

#[system(
	plugin = NpcPlugin, schedule = Update,
	after = SpawnerActivatedSet,
	in_set = EntitySpawnedSet,
)]
fn spawn_imp(
	mut ev_spawner: EventReader<SpawnerActivated>,
	mut ev_spawned: EventWriter<EntitySpawned>,
	spawners: Query<(), With<ImpSpawner>>,
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
) {
	for ev in ev_spawner.read() {
		if spawners.get(ev.spawner).is_err() {
			continue;
		}

		commands.entity(ev.entity).insert((
			Name::new("Imp"),
			EntityBundle::new(
				Transform::from_translation(ev.position),
				meshes.add(
					Capsule3d::new(0.25, 0.5)
						.mesh()
						.rings(1)
						.latitudes(8)
						.longitudes(16)
						.uv_profile(CapsuleUvProfile::Fixed),
				),
				gridbox_material("brown", &mut materials, &asset_server),
				Collider::capsule_y(0.25, 0.25),
			),
			SpawnHealthBar,
			TargetPlayer,
			RotateTowardMovement,
			Imp,
			SpawnNameTag,
		));
		ev_spawned.send(EntitySpawned(ev.entity));
	}
}
