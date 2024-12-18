#![cfg_attr(not(feature = "terminal"), windows_subsystem = "windows")]

use std::io::Cursor;

use bevy::input::common_conditions::input_just_pressed;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::winit::WinitWindows;
use bevy_rapier3d::prelude::*;
use blenvy::blueprints::spawn_from_blueprints::{
	BlueprintInfo, GameWorldTag, HideUntilReady, SpawnBlueprint,
};
use blenvy::BlenvyPlugin;
use winit::window::Icon;

use self::main_bundles::*;

mod camera;
mod entity;
mod fray;
mod gravity;
pub mod input;
mod inventory;
pub mod iter_system;
mod main_bundles;
pub mod menus;
mod npcs;
#[cfg(feature = "overview_camera")]
mod overview_camera;
mod player_commands;
mod player_controller;
mod questing;
mod skybox;
pub mod util;

fn main() {
	let mut app = App::new();
	app
		.add_plugins((
			DefaultPlugins
				.set(WindowPlugin {
					primary_window: Some(Window {
						title: "SBEPIS".to_string(),
						..default()
					}),
					..default()
				})
				.set(ImagePlugin {
					default_sampler: bevy::image::ImageSamplerDescriptor {
						address_mode_u: bevy::image::ImageAddressMode::Repeat,
						address_mode_v: bevy::image::ImageAddressMode::Repeat,
						address_mode_w: bevy::image::ImageAddressMode::Repeat,
						..default()
					},
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
			#[cfg(feature = "overview_camera")]
			overview_camera::OverviewCameraPlugin,
			BlenvyPlugin::default(),
		));

	app.add_plugins((
		player_commands::PlayerCommandsPlugin,
		camera::PlayerCameraPlugin,
		skybox::SkyboxPlugin,
		entity::EntityPlugin,
		player_controller::PlayerControllerPlugin,
		npcs::NpcPlugin,
		gravity::GravityPlugin,
		fray::FrayPlugin,
		questing::QuestingPlugin,
		menus::MenusPlugin,
		inventory::InventoryPlugin,
	))
	.add_systems(Startup, (set_window_icon, setup))
	.add_systems(
		Update,
		(
			quit.run_if(input_just_pressed(KeyCode::Escape)),
			util::despawn_after_timer,
			util::billboard,
		),
	)
	.run();
}

fn set_window_icon(windows: NonSend<WinitWindows>) {
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

fn gridbox_texture(color: &str) -> String {
	format!("Gridbox Prototype Materials/prototype_512x512_{color}.png")
}

fn gridbox_material(
	color: &str,
	materials: &mut Assets<StandardMaterial>,
	asset_server: &AssetServer,
) -> Handle<StandardMaterial> {
	gridbox_material_extra(color, materials, asset_server, StandardMaterial::default())
}

fn gridbox_material_extra(
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

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
	mut rapier_config: Query<&mut RapierConfiguration>,
) {
	let gray_material = gridbox_material("grey2", &mut materials, &asset_server);
	let green_material = gridbox_material("green1", &mut materials, &asset_server);

	commands.spawn((
		Name::new("Planet"),
		PlanetBundle::new(Vec3::Y * -1000.0, 1000.0, 10.0, &mut meshes, gray_material),
	));

	let cube_mesh = meshes.add(Cuboid::from_size(Vec3::ONE));
	commands.spawn((
		Name::new("Cube 1"),
		BoxBundle::new(
			Vec3::new(0.0, 4.0, 0.0),
			cube_mesh.clone(),
			green_material.clone(),
		),
	));
	commands.spawn((
		Name::new("Cube 2"),
		BoxBundle::new(
			Vec3::new(0.5, 5.5, 0.0),
			cube_mesh.clone(),
			green_material.clone(),
		),
	));
	commands.spawn((
		Name::new("Cube 3"),
		BoxBundle::new(
			Vec3::new(-0.5, 7.0, 0.0),
			cube_mesh.clone(),
			green_material.clone(),
		),
	));

	commands.spawn((
		Name::new("Sun"),
		DirectionalLight {
			illuminance: 4000.0,
			shadows_enabled: true,
			..default()
		},
		Transform {
			rotation: Quat::from_euler(EulerRot::XYZ, -1.9, 0.8, 0.0),
			..default()
		},
	));

	commands.spawn((
		BlueprintInfo::from_path("levels/World.glb"),
		SpawnBlueprint,
		HideUntilReady,
		GameWorldTag,
	));

	rapier_config.single_mut().gravity = Vec3::ZERO;
}

fn quit(mut ev_quit: EventWriter<AppExit>) {
	ev_quit.send(AppExit::Success);
}
