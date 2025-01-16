use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::camera::PlayerCamera;
use crate::player_controller::PlayerAction;
use crate::util::find_in_ancestors;

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
		(&mut Transform, &mut Velocity),
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
		let (mut body_transform, mut body_velocity) = player_body.single_mut();

		body_transform.rotation *= Quat::from_rotation_y(-delta.x * sensitivity.0);

		// Football imparts torque on body and LockedAxes doesn't work
		// reject_from is projection on the plane normal to the vec
		body_velocity.angvel = body_velocity
			.angvel
			.reject_from(body_transform.rotation * Vec3::Z);
	}
}

pub fn interact_with<T: Component>(
	rapier_context: Query<&RapierContext>,
	player_camera: Query<&GlobalTransform, With<PlayerCamera>>,
	entities: Query<Entity, With<T>>,
	parents: Query<&Parent>,
	input: Query<&ActionState<PlayerAction>>,
) -> Vec<Entity> {
	if !match input.iter().find(|input| !input.disabled()) {
		Some(input) => input.just_pressed(&PlayerAction::Interact),
		None => false,
	} {
		return vec![];
	}

	let player_camera = player_camera.get_single().expect("Player camera missing");
	let mut hit_entity: Option<(Option<Entity>, f32)> = None;
	rapier_context.single().intersections_with_ray(
		player_camera.translation(),
		player_camera.forward().into(),
		3.0,
		true,
		QueryFilter::default(),
		|entity, intersection| {
			if hit_entity
				.map(|(_, time)| intersection.time_of_impact < time)
				.unwrap_or(true)
				&& intersection.time_of_impact > 0.0
			{
				hit_entity = Some((
					find_in_ancestors(entity, &entities, &parents),
					intersection.time_of_impact,
				));
			}
			true
		},
	);

	if let Some((Some(entity), _)) = hit_entity {
		vec![entity]
	} else {
		vec![]
	}
}
