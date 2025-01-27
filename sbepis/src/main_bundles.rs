use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::entity::{GelViscosity, GravityOrientation, Movement};
use crate::gravity::AffectedByGravity;

#[derive(Bundle)]
pub struct BoxBundle {
	collider: Collider,
	affected_by_gravity: AffectedByGravity,
	velocity: Velocity,
	health: GelViscosity,
}

impl Default for BoxBundle {
	fn default() -> Self {
		BoxBundle {
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
}

impl BoxBundle {
	pub fn with_collider_size(self, half_size: f32) -> BoxBundle {
		BoxBundle {
			collider: Collider::cuboid(half_size, half_size, half_size),
			..self
		}
	}
}

#[derive(Bundle)]
pub struct EntityBundle {
	affected_by_gravity: AffectedByGravity,
	orientation: GravityOrientation,
	movement_input: Movement,
	locked_axes: LockedAxes,
	health: GelViscosity,
}

impl Default for EntityBundle {
	fn default() -> Self {
		EntityBundle {
			affected_by_gravity: AffectedByGravity::default(),
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
