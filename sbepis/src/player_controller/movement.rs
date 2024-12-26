use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::entity::Movement;

use super::{PlayerAction, PlayerBody};

#[derive(Resource)]
pub struct PlayerSpeed {
	pub speed: f32,
	pub sprint_modifier: f32,
	pub jump_speed: f32,
	pub friction: f32,
	pub acceleration: f32,
	pub air_acceleration: f32,
}

pub fn axes_to_ground_velocity(
	In(mut axes_input): In<Vec2>,
	key_input: Query<&ActionState<PlayerAction>>,
	mut movement: Query<(&PlayerBody, &mut Movement, &Transform)>,
	speed: Res<PlayerSpeed>,
	time: Res<Time>,
) {
	let key_input = key_input.single();
	let (body, mut movement, transform) = movement.single_mut();
	axes_input.y *= -1.;

	// Set up vectors
	let velocity = (transform.rotation.inverse() * movement.0).xz();
	let wish_velocity = axes_input
		* speed.speed
		* if key_input.pressed(&PlayerAction::Sprint) {
			speed.sprint_modifier
		} else {
			1.0
		};
	let wish_speed = wish_velocity.length();
	let wish_direction = wish_velocity.normalize_or_zero();
	let friction = if body.is_grounded {
		speed.friction
	} else {
		0.0
	};
	let acceleration = if body.is_grounded {
		speed.acceleration
	} else {
		speed.air_acceleration
	};

	// Apply friction
	let friction = if body.is_grounded {
		-time.delta_secs() * friction * velocity
	} else {
		Vec2::ZERO
	};
	let velocity = velocity + friction;

	// Do funny quake movement
	let funny_quake_speed = velocity.dot(wish_direction);
	let add_speed =
		(wish_speed - funny_quake_speed).clamp(0.0, acceleration * wish_speed * time.delta_secs()); // TODO: In absolute units, ignores relativity
	let new_velocity = velocity + wish_direction * add_speed;

	movement.0 = transform.rotation * Vec3::new(new_velocity.x, 0.0, new_velocity.y);
}

pub fn jump(
	mut player_bodies: Query<(&PlayerBody, &mut LinearVelocity, &Transform)>,
	speed: Res<PlayerSpeed>,
) {
	for (body, mut velocity, transform) in player_bodies.iter_mut() {
		if body.is_grounded {
			velocity.0 += transform.up() * speed.jump_speed;
		}
	}
}
