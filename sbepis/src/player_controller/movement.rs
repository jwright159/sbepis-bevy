use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use crate::entity::movement::AimInput;
use crate::entity::MovementInput;
use crate::netcode::ClientPlayer;
use crate::util::TransformEx;

use super::PlayerAction;

#[derive(Resource)]
pub struct PlayerSpeed {
	pub speed: f32,
	pub sprint_modifier: f32,
	pub jump_speed: f32,
}

#[derive(Component)]
pub struct PlayerHead;

#[derive(Component, Default)]
pub struct MouseAim {
	pub pitch: f32,
	pub last_pitch: f32,
	pub yaw: f32,
	pub last_yaw: f32,
}
impl MouseAim {
	pub fn pitch_delta(&mut self) -> f32 {
		let delta = self.pitch - self.last_pitch;
		self.last_pitch = self.pitch;
		delta
	}

	pub fn yaw_delta(&mut self) -> f32 {
		let delta = self.yaw - self.last_yaw;
		self.last_yaw = self.yaw;
		delta
	}
}

#[derive(Component)]
pub struct PlayerBody;

/// Probably in radians per pixel?
#[derive(Resource)]
pub struct MouseSensitivity(pub f32);

pub fn mouse_input(
	In(mouse_delta): In<Vec2>,
	mut player_aim: Query<&mut MouseAim, With<ClientPlayer>>,
	sensitivity: Res<MouseSensitivity>,
) {
	for mut aim in player_aim.iter_mut() {
		aim.yaw -= mouse_delta.x * sensitivity.0;
		aim.pitch -= mouse_delta.y * sensitivity.0;

		aim.yaw = aim.yaw.rem_euclid(2.0 * PI);
		aim.pitch = aim.pitch.clamp(-PI / 2.0, PI / 2.0);
	}
}

pub fn update_player_aim(mut aim: Query<(&mut AimInput, &mut MouseAim, &GlobalTransform)>) {
	for (mut aim_input, mut mouse_aim, transform) in aim.iter_mut() {
		aim_input.0 = Quat::from_axis_angle(transform.right().into(), mouse_aim.pitch_delta())
			* Quat::from_axis_angle(transform.up().into(), mouse_aim.yaw_delta())
			* aim_input.0;
	}
}

pub fn movement_input(
	In(mut axes_input): In<Vec2>,
	key_input: Query<&ActionState<PlayerAction>>,
	mut input: Query<(&mut MovementInput, &GlobalTransform), With<ClientPlayer>>,
	speed: Res<PlayerSpeed>,
) {
	for (mut input, transform) in input.iter_mut() {
		let key_input = key_input.single();
		axes_input.y *= -1.;
		let velocity = axes_input
			* speed.speed
			* if key_input.pressed(&PlayerAction::Sprint) {
				speed.sprint_modifier
			} else {
				1.0
			};
		input.0 = transform.transform_vector3(Vec3::new(velocity.x, 0.0, velocity.y));
	}
}

pub fn jump<Marker: Component>(
	mut player_body: Query<(&mut Velocity, &Transform), With<Marker>>,
	speed: Res<PlayerSpeed>,
) {
	for (mut velocity, transform) in player_body.iter_mut() {
		velocity.linvel += transform.up() * speed.jump_speed;
	}
}
