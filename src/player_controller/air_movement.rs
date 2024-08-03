use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::action_state::ActionState;
use super::{PlayerBody, football::PlayerSpeed, MovementAction};

pub fn air_strafe(
	In(acceleration): In<Vec2>,
	mut player_body: Query<(&mut Velocity, &Transform), With<PlayerBody>>,
	time: Res<Time>,
)
{
	let (mut velocity, transform) = player_body.single_mut();
	let delta = transform.rotation * Vec3::new(acceleration.x, 0., -acceleration.y) * time.delta_seconds();
	velocity.angvel += delta;
}

pub fn is_football_on_ground(
	football_caster: Query<&FootballGroundCaster>,
) -> bool
{
	// if football_caster.get_single().is_err() {
	// 	warn!("Football caster is empty!");
	// };
	// !football_caster.get_single().map_or(false, |hits| hits.is_empty())
	true
}

pub fn axes_to_air_acceleration(
	In(axes_input): In<Vec2>,
	input: Query<&ActionState<MovementAction>>,
	speed: Res<PlayerSpeed>,
) -> Vec2
{
	let input = input.single();
	axes_input * speed.air_acceleration * if input.pressed(&MovementAction::Sprint) { speed.sprint_modifier } else { 1.0 }
}

#[derive(Component)]
pub struct FootballGroundCaster;