use std::time::Duration;

use bevy::prelude::*;
use bevy_butler::*;
use bevy_rapier3d::prelude::*;

use crate::entity::EntityPlugin;
use crate::prelude::PlayerBody;

#[derive(Component, Deref, DerefMut, Default)]
/// The desired velocity in world-space. Will be projected onto the entity's floor plane.
pub struct Movement(pub Vec3);

#[system(
	plugin = EntityPlugin, schedule = Update,
	in_set = ExecuteMovementSet,
)]
fn strafe(mut bodies: Query<(&mut Velocity, &Transform, &Movement)>) {
	for (mut velocity, transform, input) in bodies.iter_mut() {
		velocity.linvel = velocity.linvel.project_onto(transform.up().into())
			+ input.reject_from(transform.up().into());
	}
}

#[derive(Component)]
pub struct RotateTowardMovement;

#[system(
	plugin = EntityPlugin, schedule = Update,
	in_set = ExecuteMovementSet,
)]
fn rotate_toward_movement(
	mut bodies: Query<(&mut Transform, &Movement), With<RotateTowardMovement>>,
) {
	for (mut transform, input) in bodies.iter_mut() {
		if input.length() > 0. {
			let forward = input.0.reject_from(transform.up().into());
			let up = transform.up();
			transform.look_to(forward, up);
		}
	}
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExecuteMovementSet;

#[derive(Component, Default)]
pub struct RandomInput {
	pub input: Vec3,
	pub time_since_last_change: Duration,
	pub time_to_change: Duration,
}

#[system(
	plugin = EntityPlugin, schedule = Update,
	before = ExecuteMovementSet,
)]
fn random_vec2(mut input: Query<(&mut RandomInput, &mut Movement)>, time: Res<Time>) {
	for (mut random_input, mut movement_input) in input.iter_mut() {
		random_input.time_since_last_change += time.delta();

		if random_input.time_since_last_change >= random_input.time_to_change {
			let dir = rand::random::<Vec3>().normalize() * 2.0 - Vec3::ONE;
			let mag = rand::random::<f32>() + 0.2;
			random_input.input = dir * mag;
			random_input.time_since_last_change = Duration::default();
			random_input.time_to_change =
				Duration::from_secs_f32(rand::random::<f32>() * 2.0 + 1.0);
		}

		movement_input.0 = random_input.input;
	}
}

#[derive(Component)]
pub struct TargetPlayer;

#[system(
	plugin = EntityPlugin, schedule = Update,
	before = ExecuteMovementSet,
)]
fn target_player(
	mut target_players: Query<(&Transform, &mut Movement), With<TargetPlayer>>,
	player: Query<&Transform, With<PlayerBody>>,
) {
	let player_transform = player.single();
	for (transform, mut input) in target_players.iter_mut() {
		input.0 = (player_transform.translation - transform.translation).normalize();
	}
}
