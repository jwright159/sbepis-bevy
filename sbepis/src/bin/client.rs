use bevy::input::common_conditions::input_just_pressed;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

fn main() {
	App::new()
		.insert_resource(sbepis::rapier_config())
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
			sbepis::player_commands::PlayerCommandsPlugin,
			sbepis::skybox::SkyboxPlugin,
			sbepis::entity::EntityPlugin,
			sbepis::player_controller::PlayerControllerPlugin,
			sbepis::player_controller::SpawnPlayerPlugin,
			sbepis::npcs::NpcPlugin,
			sbepis::gravity::GravityPlugin,
			sbepis::fray::FrayPlugin,
		))
		.add_systems(Startup, (sbepis::set_window_icon, sbepis::setup, sbepis::hide_mouse))
		.add_systems(
			Update,
			(
				sbepis::quit.run_if(input_just_pressed(KeyCode::Escape)),
				sbepis::util::despawn_after_timer,
				sbepis::util::billboard,
			),
		)
		.run();
}
