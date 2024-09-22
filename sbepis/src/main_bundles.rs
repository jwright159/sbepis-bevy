use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;
use bevy_rapier3d::prelude::*;

use crate::entity::movement::{AimInput, Headless};
use crate::entity::{
	GelViscosity, GravityOrientation, Healing, MovementInput, RandomInput, RotateTowardMovement,
	SpawnHealthBar, TargetPlayer,
};
use crate::gravity::{GravityPoint, GravityPriority, GravityRigidbodyBundle};
use crate::gridbox_material;
use crate::netcode::EntityType;
use crate::player_controller::{PlayerBody, WeaponSet};

#[derive(Bundle)]
pub struct PlanetBundle {
	pbr: PbrBundle,
	rigidbody: RigidBody,
	collider: Collider,
	gravity: GravityPoint,
	gravity_priority: GravityPriority,
}

impl PlanetBundle {
	pub fn new(
		position: Vec3,
		radius: f32,
		gravity: f32,
		meshes: &mut Assets<Mesh>,
		material: Handle<StandardMaterial>,
	) -> Self {
		let mut mesh = Sphere::new(radius).mesh().ico(70).unwrap();
		let uvs = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0).unwrap();
		match uvs {
			bevy::render::mesh::VertexAttributeValues::Float32x2(values) => {
				for uv in values {
					uv[0] *= radius * TAU;
					uv[1] *= radius * PI;
				}
			}
			_ => panic!("Got a UV that wasn't a Float32x2"),
		}

		let collider = Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::TriMesh)
			.expect("Couldn't make a planet collider");

		PlanetBundle {
			pbr: PbrBundle {
				transform: Transform::from_translation(position)
					.with_rotation(Quat::from_axis_angle(Vec3::X, PI / 2.)),
				mesh: meshes.add(mesh),
				material,
				..default()
			},
			rigidbody: RigidBody::Fixed,
			collider,
			gravity: GravityPoint {
				standard_radius: radius,
				acceleration_at_radius: gravity,
			},
			gravity_priority: GravityPriority(0),
		}
	}
}

#[derive(Bundle)]
pub struct BoxBundle {
	pbr: PbrBundle,
	collider: Collider,
	gravity_rigidbody_bundle: GravityRigidbodyBundle,
}

impl BoxBundle {
	pub fn new(position: Vec3, mesh: Handle<Mesh>, material: Handle<StandardMaterial>) -> Self {
		BoxBundle {
			pbr: PbrBundle {
				transform: Transform::from_translation(position),
				mesh,
				material,
				..default()
			},
			gravity_rigidbody_bundle: GravityRigidbodyBundle {
				velocity: Velocity {
					linvel: Vec3::ZERO,
					angvel: Vec3::new(2.5, 3.4, 1.6),
				},
				..default()
			},
			collider: Collider::cuboid(0.5, 0.5, 0.5),
		}
	}
}

#[derive(Bundle)]
pub struct EntityBundle {
	pbr: PbrBundle,
	gravity_rigidbody: GravityRigidbodyBundle,
	collider: Collider,
	orientation: GravityOrientation,
	movement_input: MovementInput,
	aim_input: AimInput,
	locked_axes: LockedAxes,
	health: GelViscosity,
}

impl EntityBundle {
	pub fn new(
		transform: Transform,
		mesh: Handle<Mesh>,
		material: Handle<StandardMaterial>,
		collider: Collider,
	) -> Self {
		EntityBundle {
			pbr: PbrBundle {
				transform,
				mesh,
				material,
				..default()
			},
			gravity_rigidbody: GravityRigidbodyBundle::default(),
			collider,
			orientation: GravityOrientation,
			movement_input: MovementInput::default(),
			aim_input: AimInput::default(),
			locked_axes: LockedAxes::ROTATION_LOCKED,
			health: GelViscosity {
				value: 3.0,
				max: 6.0,
			},
		}
	}
}

#[derive(Bundle)]
pub struct ConsortBundle {
	name: Name,
	entity: EntityBundle,
	spawn_health_bar: SpawnHealthBar,
	random_input: RandomInput,
	healing: Healing,
	rotate: RotateTowardMovement,
	entity_type: EntityType,
	headless: Headless,
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
			headless: Headless,
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
	headless: Headless,
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
			headless: Headless,
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

#[derive(Bundle)]
pub struct PlayerBundle {
	name: Name,
	entity: EntityBundle,
	player_body: PlayerBody,
	weapon_set: WeaponSet,
}

impl PlayerBundle {
	pub fn new(
		position: Vec3,
		meshes: &mut Assets<Mesh>,
		materials: &mut Assets<StandardMaterial>,
		asset_server: &AssetServer,
	) -> Self {
		PlayerBundle {
			name: Name::new("Player Body"),
			entity: EntityBundle::new(
				Transform::from_translation(position),
				meshes.add(
					Capsule3d::new(0.25, 1.0)
						.mesh()
						.rings(1)
						.latitudes(8)
						.longitudes(16)
						.uv_profile(CapsuleUvProfile::Fixed),
				),
				gridbox_material("white", materials, asset_server),
				Collider::capsule_y(0.5, 0.25),
			),
			player_body: PlayerBody,
			weapon_set: WeaponSet {
				weapons: vec![],
				active_weapon: None,
			},
		}
	}
}
