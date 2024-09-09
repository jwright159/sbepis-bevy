use bevy::prelude::*;

fn main() {
	App::new()
		.add_plugins((
			DefaultPlugins,
			bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
			bevy_panorbit_camera::PanOrbitCameraPlugin,
		))
		.add_systems(Startup, setup)
		.run();
}

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	commands.spawn((
		Name::new("Camera"),
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
	));

	commands.spawn((
		Name::new("Cube"),
		PbrBundle {
			mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
			material: materials.add(Color::srgb_u8(124, 144, 255)),
			..default()
		},
	));

	commands.spawn((
		Name::new("Light"),
		PointLightBundle {
			transform: Transform::from_xyz(4.0, 8.0, 4.0),
			..default()
		},
	));
}
