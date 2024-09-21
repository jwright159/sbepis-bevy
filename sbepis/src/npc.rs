use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;
use bevy_rapier3d::prelude::*;

use crate::entity::{
	GelViscosity, Healing, RandomInput, RotateTowardMovement, SpawnHealthBar, TargetPlayer,
};
use crate::netcode::EntityType;
use crate::{gridbox_material, BoxBundle, EntityBundle};

#[derive(Bundle)]
pub struct ConsortBundle {
	name: Name,
	entity: EntityBundle,
	spawn_health_bar: SpawnHealthBar,
	random_input: RandomInput,
	healing: Healing,
	rotate: RotateTowardMovement,
	entity_type: EntityType,
}
impl ConsortBundle {
	pub fn new(
		position: Vec3,
		meshes: &mut Assets<Mesh>,
		materials: &mut Assets<StandardMaterial>,
		asset_server: &AssetServer,
	) -> Self {
		ConsortBundle {
			name: Name::new("Consort"),
			entity: EntityBundle::new(
				Transform::from_translation(position),
				meshes.add(
					Capsule3d::new(0.25, 0.5)
						.mesh()
						.rings(1)
						.latitudes(8)
						.longitudes(16)
						.uv_profile(CapsuleUvProfile::Fixed),
				),
				gridbox_material("magenta", materials, asset_server),
				Collider::capsule_y(0.25, 0.25),
			),
			spawn_health_bar: SpawnHealthBar,
			random_input: RandomInput::default(),
			healing: Healing(0.2),
			rotate: RotateTowardMovement,
			entity_type: EntityType::Consort,
		}
	}
}

#[derive(Bundle)]
pub struct ImpBundle {
	name: Name,
	entity: EntityBundle,
	spawn_health_bar: SpawnHealthBar,
	target_player: TargetPlayer,
	rotate: RotateTowardMovement,
	entity_type: EntityType,
}
impl ImpBundle {
	pub fn new(
		position: Vec3,
		meshes: &mut Assets<Mesh>,
		materials: &mut Assets<StandardMaterial>,
		asset_server: &AssetServer,
	) -> Self {
		ImpBundle {
			name: Name::new("Imp"),
			entity: EntityBundle::new(
				Transform::from_translation(position),
				meshes.add(
					Capsule3d::new(0.25, 0.5)
						.mesh()
						.rings(1)
						.latitudes(8)
						.longitudes(16)
						.uv_profile(CapsuleUvProfile::Fixed),
				),
				gridbox_material("brown", materials, asset_server),
				Collider::capsule_y(0.25, 0.25),
			),
			spawn_health_bar: SpawnHealthBar,
			target_player: TargetPlayer,
			rotate: RotateTowardMovement,
			entity_type: EntityType::Imp,
		}
	}
}

#[derive(Bundle)]
pub struct CubeBundle {
	name: Name,
	box_bundle: BoxBundle,
	health: GelViscosity,
	entity_type: EntityType,
}
impl CubeBundle {
	pub fn new(
		position: Vec3,
		meshes: &mut Assets<Mesh>,
		materials: &mut Assets<StandardMaterial>,
		asset_server: &AssetServer,
	) -> Self {
		CubeBundle {
			name: Name::new("Cube"),
			box_bundle: BoxBundle::new(
				position,
				meshes.add(Cuboid::from_size(Vec3::ONE)),
				gridbox_material("green1", materials, asset_server),
			),
			health: GelViscosity {
				value: 1.0,
				max: 1.0,
			},
			entity_type: EntityType::Cube,
		}
	}
}
