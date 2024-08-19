use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::ActionState;

use super::{PlayerAction, PlayerBody};

#[derive(Resource)]
pub struct PlayerSpeed {
	pub speed: f32,
	pub sprint_modifier: f32,
	pub jump_speed: f32,
}

pub fn axes_to_ground_velocity(
	In(mut axes_input): In<Vec2>,
	input: Query<&ActionState<PlayerAction>>,
	speed: Res<PlayerSpeed>,
) -> Vec2 {
	let input = input.single();
	axes_input.y *= -1.;
	axes_input
		* speed.speed
		* if input.pressed(&PlayerAction::Sprint) {
			speed.sprint_modifier
		} else {
			1.0
		}
}

pub fn wrap_velocity_in_hashmap(
	In(speed): In<Vec2>,
	mut body: Query<Entity, With<PlayerBody>>,
) -> HashMap<Entity, Vec2> {
	let mut map = HashMap::default();
	map.insert(body.single_mut(), speed);
	map
}

pub fn strafe<Marker: Component>(
	In(speeds): In<HashMap<Entity, Vec2>>,
	mut bodies: Query<(Entity, &mut Velocity, &Transform), With<Marker>>,
) {
	for (entity, mut velocity, transform) in bodies.iter_mut() {
		let speed = match speeds.get(&entity) {
			Some(speed) => speed,
			None => continue,
		};
		let delta = transform.rotation * Vec3::new(speed.x, 0., speed.y);
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
