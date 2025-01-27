use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::entity::{GelViscosity, GravityOrientation, Movement};
use crate::gravity::AffectedByGravity;

#[derive(Component)]
#[require(
	Name(|| Name::new("Box")),
	Collider(|| Collider::cuboid(0.5, 0.5, 0.5)),
	AffectedByGravity,
	Velocity(|| Velocity {
		linvel: Vec3::ZERO,
		angvel: Vec3::new(2.5, 3.4, 1.6),
	}),
	GelViscosity(|| GelViscosity {
		value: 1.0,
		max: 1.0,
	}),
)]
pub struct Box;

#[derive(Component)]
#[require(
	Name(|| Name::new("Mob")),
	AffectedByGravity,
	GravityOrientation,
	Movement,
	LockedAxes(|| LockedAxes::ROTATION_LOCKED),
	GelViscosity(|| GelViscosity {
		value: 3.0,
		max: 6.0,
	}),
	Visibility,
)]
pub struct Mob;
