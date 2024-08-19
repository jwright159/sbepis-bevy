use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::player_controller::{GravityOrientation, Health, MovementInput};

use super::gravity::*;

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
	health: Health,
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
			health: Health(5.0),
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
	locked_axes: LockedAxes,
	health: Health,
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
			locked_axes: LockedAxes::ROTATION_LOCKED,
			health: Health(3.0),
		}
	}
}
