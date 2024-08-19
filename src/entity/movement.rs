use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component, Deref, DerefMut, Default)]
pub struct MovementInput(pub Vec2);

pub fn strafe(mut bodies: Query<(&mut Velocity, &Transform, &MovementInput)>) {
	for (mut velocity, transform, input) in bodies.iter_mut() {
		let delta = transform.rotation * Vec3::new(input.x, 0., input.y);
		velocity.linvel = velocity.linvel.project_onto(transform.up().into()) + delta;
	}
}
