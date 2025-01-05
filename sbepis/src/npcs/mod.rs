use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_rapier3d::geometry::Collider;
use name_tags::*;

use crate::entity::spawner::{spawn_entities, SpawnEntityInformation, SpawnedEntity};
use crate::entity::{Healing, RandomInput, RotateTowardMovement, SpawnHealthBar, TargetPlayer};
use crate::main_bundles::EntityBundle;
use crate::questing::{QuestGiver, SpawnQuestMarker};
use crate::{gridbox_material, some_or_return};

mod name_tags;

pub struct NpcPlugin;
impl Plugin for NpcPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(RonAssetPlugin::<AvailableNames>::new(&["names.ron"]))
			.init_resource::<FontMeshGenerator>()
			.add_systems(Startup, (load_names,))
			.add_systems(
				Update,
				(
					spawn_entities::<ConsortSpawner, Consort>.pipe(spawn_consort),
					spawn_entities::<ImpSpawner, Imp>.pipe(spawn_imp),
					spawn_name_tags,
				),
			);
	}
}

#[derive(Component)]
pub struct Consort;

#[derive(Component)]
pub struct ConsortSpawner;

#[derive(Component)]
pub struct Imp;

#[derive(Component)]
pub struct ImpSpawner;

fn spawn_consort(
	In(spawn_info): In<Option<SpawnEntityInformation>>,
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
) {
	let spawn_info = some_or_return!(spawn_info);
	commands.spawn((
		Name::new("Consort"),
		EntityBundle::new(
			Transform::from_translation(spawn_info.position),
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
		SpawnedEntity {
			spawner: spawn_info.spawner,
		},
		Consort,
		QuestGiver::default(),
		SpawnQuestMarker,
		SpawnNameTag,
	));
}

fn spawn_imp(
	In(spawn_info): In<Option<SpawnEntityInformation>>,
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
) {
	let spawn_info = some_or_return!(spawn_info);
	commands.spawn((
		Name::new("Imp"),
		EntityBundle::new(
			Transform::from_translation(spawn_info.position),
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
		SpawnedEntity {
			spawner: spawn_info.spawner,
		},
		Imp,
		SpawnNameTag,
	));
}
