use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use super::{MovementAction, PlayerBody};

#[derive(Component)]
pub struct Football
{
	pub radius: f32,
}

#[derive(Component)]
pub struct FootballJoint
{
	pub rest_local_position: Vec3,
	pub jump_local_position: Vec3,
	pub jump_speed: f32,
}

#[derive(Resource)]
pub struct PlayerSpeed
{
	pub speed: f32,
	pub sprint_modifier: f32,
	pub air_acceleration: f32,
}

pub fn axes_to_ground_velocity(
	In(axes_input): In<Vec2>,
	input: Query<&ActionState<MovementAction>>,
	speed: Res<PlayerSpeed>,
) -> Vec2
{
	let input = input.single();
	axes_input * speed.speed * if input.pressed(&MovementAction::Sprint) { speed.sprint_modifier } else { 1.0 }
}

pub fn spin_football(
	In(input_velocity): In<Vec2>,
	mut football: Query<(&mut Velocity, &Football), Without<PlayerBody>>,
	player_body: Query<&Transform, With<PlayerBody>>,
)
{
	let (mut velocity, football) = football.single_mut();
	let body_transform = player_body.single();
	velocity.angvel = body_transform.rotation * Vec3::new(-input_velocity.y, 0., -input_velocity.x) / football.radius;
}

pub fn jump(
	In(is_jumping): In<bool>,
	mut football_joint: Query<(&mut ImpulseJoint, &FootballJoint)>,
	time: Res<Time>,
)
{
	for (mut joint, joint_params) in football_joint.iter_mut() {
		if let TypedJoint::SphericalJoint(joint) = &mut joint.data {
			let target = if is_jumping { joint_params.jump_local_position } else { joint_params.rest_local_position };
			joint.set_local_anchor1(joint.local_anchor1() + (target - joint.local_anchor1()).clamp_length_max(time.delta_seconds() * joint_params.jump_speed));
		}
	}
}