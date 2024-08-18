use super::{PlayerAction, PlayerBody};
use bevy::prelude::*;
use bevy_rapier3d::prelude::{Real, *};
use leafwing_input_manager::prelude::ActionState;

#[derive(Resource)]
pub struct PlayerSpeed {
	pub speed: Real,
	pub sprint_modifier: Real,
	pub jump_speed: Real,
}

pub fn axes_to_ground_velocity(
	In(axes_input): In<Vec2>,
	input: Query<&ActionState<PlayerAction>>,
	speed: Res<PlayerSpeed>,
) -> Vec2 {
	let input = input.single();
	axes_input
		* speed.speed
		* if input.pressed(&PlayerAction::Sprint) {
			speed.sprint_modifier
		} else {
			1.0
		}
}

pub fn strafe(
	In(speed): In<Vec2>,
	mut player_body: Query<(&mut Velocity, &Transform), With<PlayerBody>>,
) {
	let (mut velocity, transform) = player_body.single_mut();
	let delta = transform.rotation * Vec3::new(speed.x, 0., -speed.y);
	velocity.linvel = velocity.linvel.project_onto(transform.up().into()) + delta;
}

pub fn jump(
	mut player_body: Query<(&mut Velocity, &Transform), With<PlayerBody>>,
	speed: Res<PlayerSpeed>,
) {
	let (mut velocity, transform) = player_body.single_mut();
	velocity.linvel += transform.up() * speed.jump_speed;
}
