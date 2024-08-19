use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use super::{PlayerAction, PlayerBody};

#[derive(Resource)]
pub struct PlayerSpeed {
	pub speed: f32,
	pub sprint_modifier: f32,
	pub jump_speed: f32,
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct MovementInput(pub Vec2);

pub fn axes_to_ground_velocity(
	In(mut axes_input): In<Vec2>,
	key_input: Query<&ActionState<PlayerAction>>,
	mut input: Query<&mut MovementInput, With<PlayerBody>>,
	speed: Res<PlayerSpeed>,
) {
	let key_input = key_input.single();
	let mut input = input.single_mut();
	axes_input.y *= -1.;
	input.0 = axes_input
		* speed.speed
		* if key_input.pressed(&PlayerAction::Sprint) {
			speed.sprint_modifier
		} else {
			1.0
		};
}

pub fn strafe(mut bodies: Query<(&mut Velocity, &Transform, &MovementInput)>) {
	for (mut velocity, transform, input) in bodies.iter_mut() {
		let delta = transform.rotation * Vec3::new(input.x, 0., input.y);
		velocity.linvel = velocity.linvel.project_onto(transform.up().into()) + delta;
	}
}

pub fn jump<Marker: Component>(
	mut player_body: Query<(&mut Velocity, &Transform), With<Marker>>,
	speed: Res<PlayerSpeed>,
) {
	let (mut velocity, transform) = player_body.single_mut();
	velocity.linvel += transform.up() * speed.jump_speed;
}
