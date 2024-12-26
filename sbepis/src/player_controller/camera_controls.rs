use std::f32::consts::PI;

use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::camera::PlayerCamera;
use crate::player_controller::PlayerAction;

#[derive(Component)]
pub struct Pitch(pub f32);

#[derive(Component)]
pub struct PlayerBody {
	pub is_grounded: bool,
}

/// Probably in radians per pixel?
#[derive(Resource)]
pub struct MouseSensitivity(pub f32);

pub fn rotate_camera_and_body(
	In(delta): In<Vec2>,
	sensitivity: Res<MouseSensitivity>,
	mut player_camera: Query<
		(&mut Transform, &mut Pitch, &Camera),
		(With<PlayerCamera>, Without<PlayerBody>),
	>,
	mut player_body: Query<
		(&mut Transform, &mut AngularVelocity),
		(Without<PlayerCamera>, With<PlayerBody>),
	>,
) {
	{
		let (mut camera_transform, mut camera_pitch, camera) = player_camera.single_mut();
		if !camera.is_active {
			return;
		}

		camera_pitch.0 += delta.y * sensitivity.0;
		camera_pitch.0 = camera_pitch.0.clamp(-PI / 2., PI / 2.);
		camera_transform.rotation = Quat::from_rotation_x(-camera_pitch.0);
	}

	{
		let (mut body_transform, mut body_angular_velocity) = player_body.single_mut();

		body_transform.rotation *= Quat::from_rotation_y(-delta.x * sensitivity.0);

		// Football imparts torque on body and LockedAxes doesn't work
		// reject_from is projection on the plane normal to the vec
		body_angular_velocity.0 = body_angular_velocity
			.0
			.reject_from(body_transform.rotation * Vec3::Z);
	}
}

pub fn interact_with<T: Component>(
	ray_hits: Query<&RayHits, With<PlayerCamera>>,
	entities: Query<(), With<T>>,
	input: Query<&ActionState<PlayerAction>>,
) -> Vec<Option<Entity>> {
	if !match input.iter().find(|input| !input.disabled()) {
		Some(input) => input.just_pressed(&PlayerAction::Interact),
		None => false,
	} {
		return vec![];
	}

	let ray_hits = ray_hits.get_single().expect("Ray hits missing");
	vec![ray_hits
		.iter()
		.next()
		.map(|hit| hit.entity)
		.filter(|entity| entities.get(*entity).is_ok())]
}
