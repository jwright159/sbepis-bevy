use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy_butler::*;

use crate::camera::PlayerCamera;
use crate::menus::{
	Menu, MenuActivated, MenuActivatedSet, MenuDeactivated, MenuDeactivatedSet,
	MenuManipulationSet, MenuStack, MenuWithMouse,
};

#[butler_plugin(build(add_plugins(bevy_panorbit_camera::PanOrbitCameraPlugin)))]
pub struct OverviewCameraPlugin;

#[derive(Component)]
pub struct OverviewCamera;

#[system(
	plugin = OverviewCameraPlugin, schedule = Startup,
)]
fn setup(mut commands: Commands) {
	commands.spawn((
		Name::new("Overview Camera"),
		Camera {
			is_active: false,
			..default()
		},
		Transform::from_xyz(4.0, 6.5, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
		bevy_panorbit_camera::PanOrbitCamera {
			button_orbit: MouseButton::Left,
			button_pan: MouseButton::Left,
			modifier_pan: Some(KeyCode::ShiftLeft),
			reversed_zoom: true,
			..default()
		},
		OverviewCamera,
		Menu,
		MenuWithMouse,
	));
}

#[system(
	plugin = OverviewCameraPlugin, schedule = Update,
	run_if = input_just_pressed(KeyCode::Tab),
	in_set = MenuManipulationSet,
)]
fn toggle_camera(
	mut menu_stack: ResMut<MenuStack>,
	overview_camera: Query<Entity, With<OverviewCamera>>,
) {
	let overview_camera = overview_camera
		.get_single()
		.expect("No overview camera found");
	menu_stack.toggle(overview_camera);
}

#[system(
	plugin = OverviewCameraPlugin, schedule = Update,
	after = MenuActivatedSet,
)]
fn enable_overview_camera(
	mut ev_activated: EventReader<MenuActivated>,
	mut overview_camera: Query<&mut Camera, (With<OverviewCamera>, Without<PlayerCamera>)>,
	mut player_camera: Query<&mut Camera, (With<PlayerCamera>, Without<OverviewCamera>)>,
) {
	for MenuActivated(menu) in ev_activated.read() {
		if overview_camera.get(*menu).is_ok() {
			overview_camera.single_mut().is_active = true;
			player_camera.single_mut().is_active = false;
		}
	}
}

#[system(
	plugin = OverviewCameraPlugin, schedule = Update,
	after = MenuDeactivatedSet,
)]
fn disable_overview_camera(
	mut ev_deactivated: EventReader<MenuDeactivated>,
	mut overview_camera: Query<&mut Camera, (With<OverviewCamera>, Without<PlayerCamera>)>,
	mut player_camera: Query<&mut Camera, (With<PlayerCamera>, Without<OverviewCamera>)>,
) {
	for MenuDeactivated(menu) in ev_deactivated.read() {
		if overview_camera.get(*menu).is_ok() {
			overview_camera.single_mut().is_active = false;
			player_camera.single_mut().is_active = true;
		}
	}
}
