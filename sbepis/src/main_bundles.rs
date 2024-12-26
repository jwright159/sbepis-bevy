use std::f32::consts::{PI, TAU};

use avian3d::prelude::*;
use bevy::prelude::*;

use crate::entity::{GelViscosity, GravityOrientation, Movement};
use crate::gravity::{AffectedByGravity, GravityPoint, GravityPriority};
use crate::util::CreateMeshCollider;

#[derive(Bundle)]
pub struct PlanetBundle {
	transform: Transform,
	rigidbody: RigidBody,
	collider: CreateMeshCollider,
	gravity: GravityPoint,
	gravity_priority: GravityPriority,
	scene: SceneRoot,
}

impl PlanetBundle {
	pub fn new(position: Vec3, radius: f32, gravity: f32, asset_server: &AssetServer) -> Self {
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

		PlanetBundle {
			transform: Transform::from_translation(position)
				.with_rotation(Quat::from_axis_angle(Vec3::X, PI / 2.)),
			rigidbody: RigidBody::Static,
			collider: CreateMeshCollider,
			gravity: GravityPoint {
				standard_radius: radius,
				acceleration_at_radius: gravity,
			},
			gravity_priority: GravityPriority(0),
			scene: SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("planet.glb"))),
		}
	}
}

#[derive(Bundle)]
pub struct BoxBundle {
	transform: Transform,
	mesh: Mesh3d,
	material: MeshMaterial3d<StandardMaterial>,
	collider: Collider,
	affected_by_gravity: AffectedByGravity,
	linear_velocity: LinearVelocity,
	angular_velocity: AngularVelocity,
	health: GelViscosity,
}

impl BoxBundle {
	pub fn new(position: Vec3, mesh: Handle<Mesh>, material: Handle<StandardMaterial>) -> Self {
		BoxBundle {
			transform: Transform::from_translation(position),
			mesh: Mesh3d(mesh),
			material: MeshMaterial3d(material),
			affected_by_gravity: AffectedByGravity::default(),
			linear_velocity: LinearVelocity(Vec3::new(0.0, 0.0, 0.0)),
			angular_velocity: AngularVelocity(Vec3::new(2.5, 3.4, 1.6)),
			collider: Collider::cuboid(1.0, 1.0, 1.0),
			health: GelViscosity {
				value: 1.0,
				max: 1.0,
			},
		}
	}

	pub fn with_collider_size(self, half_size: f32) -> BoxBundle {
		BoxBundle {
			collider: Collider::cuboid(half_size, half_size, half_size),
			..self
		}
	}
}

#[derive(Bundle)]
pub struct EntityBundle {
	transform: Transform,
	mesh: Mesh3d,
	material: MeshMaterial3d<StandardMaterial>,
	affected_by_gravity: AffectedByGravity,
	collider: Collider,
	orientation: GravityOrientation,
	movement_input: Movement,
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
			transform,
			mesh: Mesh3d(mesh),
			material: MeshMaterial3d(material),
			affected_by_gravity: AffectedByGravity::default(),
			collider,
			orientation: GravityOrientation,
			movement_input: Movement::default(),
			locked_axes: LockedAxes::ROTATION_LOCKED,
			health: GelViscosity {
				value: 3.0,
				max: 6.0,
			},
		}
	}
}
