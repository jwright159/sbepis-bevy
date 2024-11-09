use bevy::prelude::*;

use crate::ok_or_return;

pub struct PlayerCameraPlugin;
impl Plugin for PlayerCameraPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			Update,
			(
				setup_player_camera_added_node,
				setup_player_camera_added_camera,
			),
		);
	}
}

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Component)]
pub struct PlayerCameraNode;

pub fn setup_player_camera_added_node(
	mut commands: Commands,
	nodes: Query<Entity, Added<PlayerCameraNode>>,
	camera: Query<Entity, With<PlayerCamera>>,
) {
	let camera = ok_or_return!(camera.get_single());
	for node in nodes.iter() {
		commands.entity(node).insert(TargetCamera(camera));
	}
}

pub fn setup_player_camera_added_camera(
	mut commands: Commands,
	nodes: Query<Entity, With<PlayerCameraNode>>,
	camera: Query<Entity, Added<PlayerCamera>>,
) {
	let camera = ok_or_return!(camera.get_single());
	for node in nodes.iter() {
		commands.entity(node).insert(TargetCamera(camera));
	}
}
