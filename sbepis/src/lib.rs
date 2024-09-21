#![cfg_attr(not(feature = "terminal"), windows_subsystem = "windows")]

use std::io::Cursor;

use bevy::input::common_conditions::input_just_pressed;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy::winit::WinitWindows;
use bevy_rapier3d::prelude::*;
use winit::window::Icon;

use self::main_bundles::*;

pub mod entity;
pub mod fray;
pub mod gravity;
pub mod input;
pub mod main_bundles;
pub mod netcode;
pub mod overview_camera;
pub mod player_commands;
pub mod player_controller;
pub mod skybox;
pub mod util;

pub struct CommonPlugin {
	window_name: String,
}
impl Plugin for CommonPlugin {
	fn build(&self, app: &mut App) {
		app
			.insert_resource(rapier_config())
			.add_plugins((
				DefaultPlugins
					.set(WindowPlugin {
						primary_window: Some(Window {
							title: self.window_name.clone(),
							..default()
						}),
						..default()
					})
					.set(ImagePlugin {
						default_sampler: bevy::render::texture::ImageSamplerDescriptor {
							address_mode_u: bevy::render::texture::ImageAddressMode::Repeat,
							address_mode_v: bevy::render::texture::ImageAddressMode::Repeat,
							address_mode_w: bevy::render::texture::ImageAddressMode::Repeat,
							..default()
						}
						.into(),
					})
					.set(LogPlugin {
						filter: "info,sbepis=debug,avian3d=debug,wgpu=error,naga=warn,calloop=error,symphonia_core=warn,symphonia_bundle_mp3=warn".into(),
						..default()
					}),
				RapierPhysicsPlugin::<NoUserData>::default(),
				#[cfg(feature = "rapier_debug")]
				RapierDebugRenderPlugin::default(),
				#[cfg(feature = "inspector")]
				bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
				player_commands::PlayerCommandsPlugin,
				skybox::SkyboxPlugin,
				entity::EntityPlugin,
				player_controller::PlayerControllerPlugin,
				gravity::GravityPlugin,
				fray::FrayPlugin,
			))
			.add_systems(Startup, (set_window_icon, setup_world))
			.add_systems(
				Update,
				(
					quit.run_if(input_just_pressed(KeyCode::Escape)),
					util::despawn_after_timer,
					util::billboard,
				),
			);
	}
}
impl CommonPlugin {
	pub fn new(window_name: &str) -> Self {
		Self {
			window_name: window_name.to_string(),
		}
	}
}

pub fn rapier_config() -> RapierConfiguration {
	let mut rapier_config = RapierConfiguration::new(1.);
	rapier_config.gravity = Vec3::ZERO;
	rapier_config
}

pub fn set_window_icon(windows: NonSend<WinitWindows>) {
	let icon_buf = Cursor::new(include_bytes!("../assets/house.png"));
	let image = image::load(icon_buf, image::ImageFormat::Png).unwrap();
	let image = image.into_rgba8();
	let (width, height) = image.dimensions();
	let rgba = image.into_raw();
	let icon = Icon::from_rgba(rgba, width, height).unwrap();

	for window in windows.windows.values() {
		window.set_window_icon(Some(icon.clone()));
	}
}

pub fn gridbox_texture(color: &str) -> String {
	format!("Gridbox Prototype Materials/prototype_512x512_{color}.png")
}

pub fn gridbox_material(
	color: &str,
	materials: &mut Assets<StandardMaterial>,
	asset_server: &AssetServer,
) -> Handle<StandardMaterial> {
	gridbox_material_extra(color, materials, asset_server, StandardMaterial::default())
}

pub fn gridbox_material_extra(
	color: &str,
	materials: &mut Assets<StandardMaterial>,
	asset_server: &AssetServer,
	material: StandardMaterial,
) -> Handle<StandardMaterial> {
	materials.add(StandardMaterial {
		base_color_texture: Some(asset_server.load(gridbox_texture(color))),
		..material
	})
}

pub fn setup_world(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
) {
	let gray_material = gridbox_material("grey2", &mut materials, &asset_server);

	commands.spawn((
		Name::new("Planet"),
		PlanetBundle::new(Vec3::Y * -1000.0, 1000.0, 10.0, &mut meshes, gray_material),
	));

	commands.spawn((
		Name::new("Sun"),
		DirectionalLightBundle {
			directional_light: DirectionalLight {
				illuminance: 4000.0,
				shadows_enabled: true,
				..default()
			},
			transform: Transform {
				rotation: Quat::from_euler(EulerRot::XYZ, -1.9, 0.8, 0.0),
				..default()
			},
			..default()
		},
	));
}

pub fn hide_mouse(mut window: Query<&mut Window, With<PrimaryWindow>>) {
	let mut window = window.single_mut();
	window.cursor.grab_mode = CursorGrabMode::Locked;
	window.cursor.visible = false;
}

pub fn quit(mut ev_quit: EventWriter<AppExit>) {
	ev_quit.send(AppExit::Success);
}
