use std::net::UdpSocket;
use std::time::SystemTime;

use bevy::prelude::*;
use bevy_renet::renet::transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_renet::renet::{RenetServer, ServerEvent};
use bevy_renet::transport::NetcodeServerPlugin;
use bevy_renet::RenetServerPlugin;
use sbepis::netcode::*;

fn main() {
	App::new()
		.add_plugins((
			sbepis::CommonPlugin::new("SBEPIS Server"),
			ServerPlugin,
			sbepis::overview_camera::OverviewCameraPlugin,
		))
		.add_systems(Startup, setup)
		.run();
}

fn setup(mut commands: Commands, mut server_commands: EventWriter<ServerCommand>) {
	server_commands.send(ServerCommand::SpawnEntity(
		commands.spawn_empty().id(),
		EntityType::Cube,
		Vec3::new(0.0, 4.0, 0.0),
	));
	server_commands.send(ServerCommand::SpawnEntity(
		commands.spawn_empty().id(),
		EntityType::Cube,
		Vec3::new(0.5, 5.5, 0.0),
	));
	server_commands.send(ServerCommand::SpawnEntity(
		commands.spawn_empty().id(),
		EntityType::Cube,
		Vec3::new(-0.5, 7.0, 0.0),
	));

	server_commands.send(ServerCommand::SpawnEntity(
		commands.spawn_empty().id(),
		EntityType::Consort,
		Vec3::new(0.0, 10.0, 0.0),
	));
	server_commands.send(ServerCommand::SpawnEntity(
		commands.spawn_empty().id(),
		EntityType::Imp,
		Vec3::new(-6.0, 10.0, 0.0),
	));
}

struct ServerPlugin;
impl Plugin for ServerPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(RenetServerPlugin);
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

		app.add_systems(Update, (server_send, server_recieve));
	}
}

fn server_send(
	mut server: ResMut<RenetServer>,
	mut commands: EventReader<ServerCommand>,
	mut server_events: EventReader<ServerEvent>,
	entities: Query<(Entity, &GlobalTransform, &EntityType)>,
) {
	for event in server_events.read() {
		match event {
			ServerEvent::ClientConnected { client_id } => {
				info!("Client connected: {}", client_id);
				for (entity, transform, entity_type) in entities.iter() {
					server.send_message(
						*client_id,
						ServerChannel::Commands,
						bincode::serialize(&ServerCommand::SpawnEntity(
							entity,
							*entity_type,
							transform.translation(),
						))
						.unwrap(),
					);
				}
			}
			ServerEvent::ClientDisconnected { client_id, reason } => {
				info!("Client disconnected: {} because {}", client_id, reason);
			}
		}
	}

	for command in commands.read() {
		let command = bincode::serialize(&command).unwrap();
		server.broadcast_message(ServerChannel::Commands, command);
	}
}

fn server_recieve(mut server: ResMut<RenetServer>) {
	for client_id in server.clients_id() {
		while let Some(message) = server.receive_message(client_id, ClientChannel::Commands) {
			let server_message: Vec3 = bincode::deserialize(&message).unwrap();
			println!("Recieved {}", server_message);
		}
	}
}
