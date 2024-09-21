use bevy::prelude::*;

pub struct OverviewCameraPlugin;

impl Plugin for OverviewCameraPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((bevy_panorbit_camera::PanOrbitCameraPlugin,))
			.add_systems(Startup, (setup,));
	}
}

#[derive(Component)]
pub struct OverviewCamera;

fn setup(mut commands: Commands) {
	commands.spawn((
		Name::new("Overview Camera"),
		Camera3dBundle {
			transform: Transform::from_xyz(4.0, 6.5, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
			..default()
		},
		bevy_panorbit_camera::PanOrbitCamera {
			button_orbit: MouseButton::Left,
			button_pan: MouseButton::Left,
			modifier_pan: Some(KeyCode::ShiftLeft),
			reversed_zoom: true,
			..default()
		},
		OverviewCamera,
	));
}
