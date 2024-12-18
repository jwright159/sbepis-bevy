use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::entity::{GelViscosity, GravityOrientation, MovementInput};
use crate::gravity::{AffectedByGravity, GravityPoint, GravityPriority};

// TODO: Move this stuff to blenvy

#[derive(Bundle)]
pub struct PlanetBundle {
	transform: Transform,
	mesh: Mesh3d,
	material: MeshMaterial3d<StandardMaterial>,
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

		let collider = Collider::from_bevy_mesh(&mesh, &ComputedColliderShape::default())
			.expect("Couldn't make a planet collider");

		PlanetBundle {
			transform: Transform::from_translation(position)
				.with_rotation(Quat::from_axis_angle(Vec3::X, PI / 2.)),
			mesh: Mesh3d(meshes.add(mesh)),
			material: MeshMaterial3d(material),
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
	transform: Transform,
	mesh: Mesh3d,
	material: MeshMaterial3d<StandardMaterial>,
	collider: Collider,
	affected_by_gravity: AffectedByGravity,
	velocity: Velocity,
	health: GelViscosity,
}

impl BoxBundle {
	pub fn new(position: Vec3, mesh: Handle<Mesh>, material: Handle<StandardMaterial>) -> Self {
		BoxBundle {
			transform: Transform::from_translation(position),
			mesh: Mesh3d(mesh),
			material: MeshMaterial3d(material),
			affected_by_gravity: AffectedByGravity::default(),
			velocity: Velocity {
				linvel: Vec3::ZERO,
				angvel: Vec3::new(2.5, 3.4, 1.6),
			},
			collider: Collider::cuboid(0.5, 0.5, 0.5),
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
	movement_input: MovementInput,
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
			movement_input: MovementInput::default(),
			locked_axes: LockedAxes::ROTATION_LOCKED,
			health: GelViscosity {
				value: 3.0,
				max: 6.0,
			},
		}
	}
}
