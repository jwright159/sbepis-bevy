use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::entity::MovementInput;

use super::{PlayerAction, PlayerBody};

#[derive(Resource)]
pub struct PlayerSpeed {
	pub speed: f32,
	pub sprint_modifier: f32,
	pub jump_speed: f32,
}

pub fn axes_to_ground_velocity(
	In(mut axes_input): In<Vec2>,
	key_input: Query<&ActionState<PlayerAction>>,
	mut input: Query<(&mut MovementInput, &Transform), With<PlayerBody>>,
	speed: Res<PlayerSpeed>,
) {
	let key_input = key_input.single();
	let (mut input, transform) = input.single_mut();
	axes_input.y *= -1.;
	let velocity = axes_input
		* speed.speed
		* if key_input.pressed(&PlayerAction::Sprint) {
			speed.sprint_modifier
		} else {
			1.0
		};
	input.0 = transform.rotation * Vec3::new(velocity.x, 0.0, velocity.y);
}

pub fn jump<Marker: Component>(
	mut player_body: Query<(&mut LinearVelocity, &Transform), With<Marker>>,
	speed: Res<PlayerSpeed>,
) {
	let (mut velocity, transform) = player_body.single_mut();
	velocity.0 += transform.up() * speed.jump_speed;
}
