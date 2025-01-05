use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::entity::{GelViscosity, GravityOrientation, Movement};
use crate::gravity::AffectedByGravity;

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
