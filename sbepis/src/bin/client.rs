use bevy::input::common_conditions::input_just_pressed;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_renet::client_connected;
use bevy_renet::renet::RenetClient;
use sbepis::netcode::*;

fn main() {
	let mut app = App::new();
	app
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
		.add_systems(
			Update,
			(client_send, client_recieve).in_set(Connected),
		);

	add_netcode_network(&mut app);

	app.run();
}

#[derive(Debug, Resource)]
struct CurrentClientId(u64);

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Connected;

fn add_netcode_network(app: &mut App) {
	use bevy_renet::renet::transport::{
		ClientAuthentication, NetcodeClientTransport, NetcodeTransportError,
	};
	use std::{net::UdpSocket, time::SystemTime};

	app.add_plugins(bevy_renet::transport::NetcodeClientPlugin);

	app.configure_sets(Update, Connected.run_if(client_connected));

	let client = RenetClient::new(connection_config());

	let server_addr = "127.0.0.1:5000".parse().unwrap();
	let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
	let current_time = SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.unwrap();
	let client_id = current_time.as_millis() as u64;
	let authentication = ClientAuthentication::Unsecure {
		client_id,
		protocol_id: PROTOCOL_ID,
		server_addr,
		user_data: None,
	};

	let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

	app.insert_resource(client);
	app.insert_resource(transport);
	app.insert_resource(CurrentClientId(client_id));

	// If any error is found we just panic
	#[allow(clippy::never_loop)]
	fn panic_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
		for e in renet_error.read() {
			panic!("{}", e);
		}
	}

	app.add_systems(Update, panic_on_error_system);
}

fn client_send(mut client: ResMut<RenetClient>) {
	let input_message = bincode::serialize(&Vec3::X).unwrap();
	client.send_message(ClientChannel::Command, input_message);
}

fn client_recieve(mut client: ResMut<RenetClient>) {
	while let Some(message) = client.receive_message(ServerChannel::ServerMessages) {
		let server_message: Vec3 = bincode::deserialize(&message).unwrap();
		println!("Recieved {}", server_message);
	}
}
