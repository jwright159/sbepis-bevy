use bevy::input::common_conditions::input_just_pressed;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_renet::renet::RenetServer;
use sbepis::netcode::*;

fn main() {
	let mut app = App::new();
	app
		.insert_resource(sbepis::rapier_config())
		.add_plugins((
			DefaultPlugins
				.set(WindowPlugin {
					primary_window: Some(Window {
						title: "SBEPIS Server".to_string(),
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
			sbepis::overview_camera::OverviewCameraPlugin,
			sbepis::player_commands::PlayerCommandsPlugin,
			sbepis::skybox::SkyboxPlugin,
			sbepis::entity::EntityPlugin,
			sbepis::player_controller::PlayerControllerPlugin,
			sbepis::npcs::NpcPlugin,
			sbepis::gravity::GravityPlugin,
			sbepis::fray::FrayPlugin,
		))
		.add_systems(Startup, (sbepis::set_window_icon, sbepis::setup))
		.add_systems(
			Update,
			(
				sbepis::quit.run_if(input_just_pressed(KeyCode::Escape)),
				sbepis::util::despawn_after_timer,
				sbepis::util::billboard,
			),
		)
		.add_systems(Update, (server_send,
            server_recieve));

	add_netcode_network(&mut app);

	app.run();
}

fn add_netcode_network(app: &mut App) {
	use bevy_renet::renet::transport::{
		NetcodeServerTransport, ServerAuthentication, ServerConfig,
	};
	use bevy_renet::transport::NetcodeServerPlugin;
	use std::{net::UdpSocket, time::SystemTime};

	app.add_plugins(NetcodeServerPlugin);

	let server = RenetServer::new(connection_config());

	let public_addr = "127.0.0.1:5000".parse().unwrap();
	let socket = UdpSocket::bind(public_addr).unwrap();
	let current_time: std::time::Duration = SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.unwrap();
	let server_config = ServerConfig {
		current_time,
		max_clients: 64,
		protocol_id: PROTOCOL_ID,
		public_addresses: vec![public_addr],
		authentication: ServerAuthentication::Unsecure,
	};

	let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
	app.insert_resource(server);
	app.insert_resource(transport);
}

fn server_send(mut server: ResMut<RenetServer>) {
	let sync_message = bincode::serialize(&Vec3::Y).unwrap();
	server.broadcast_message(ServerChannel::ServerMessages, sync_message);
}

fn server_recieve(mut server: ResMut<RenetServer>) {
	for client_id in server.clients_id() {
		while let Some(message) = server.receive_message(client_id, ClientChannel::Command) {
			let server_message: Vec3 = bincode::deserialize(&message).unwrap();
			println!("Recieved {}", server_message);
		}
	}
}
