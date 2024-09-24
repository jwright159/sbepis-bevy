use std::net::UdpSocket;
use std::time::SystemTime;

use bevy::prelude::*;
use bevy_renet::renet::transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_renet::renet::{RenetServer, ServerEvent};
use bevy_renet::transport::NetcodeServerPlugin;
use bevy_renet::RenetServerPlugin;
use sbepis::entity::RandomInput;
use sbepis::netcode::*;
use sbepis::player_controller::PlayerBody;
use serde::Serialize;

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
		ServerEntity(commands.spawn_empty().id()),
		EntityType::Cube,
		Vec3::new(0.0, 4.0, 0.0),
	));
	server_commands.send(ServerCommand::SpawnEntity(
		ServerEntity(commands.spawn_empty().id()),
		EntityType::Cube,
		Vec3::new(0.5, 5.5, 0.0),
	));
	server_commands.send(ServerCommand::SpawnEntity(
		ServerEntity(commands.spawn_empty().id()),
		EntityType::Cube,
		Vec3::new(-0.5, 7.0, 0.0),
	));

	server_commands.send(ServerCommand::SpawnEntity(
		ServerEntity(commands.spawn_empty().id()),
		EntityType::Consort,
		Vec3::new(0.0, 10.0, 0.0),
	));
	server_commands.send(ServerCommand::SpawnEntity(
		ServerEntity(commands.spawn_empty().id()),
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
		app.insert_resource(ServerState::Server);

		app.add_systems(
			Update,
			(
				server_events,
				server_send,
				server_recieve,
				server_send_component::<RandomInput>,
				server_send_component_entity_not_player::<Transform>,
			),
		);
	}
}

fn server_events(
	mut server: ResMut<RenetServer>,
	mut commands: Commands,
	mut server_commands: EventWriter<ServerCommand>,
	mut server_events: EventReader<ServerEvent>,
	entities: Query<(Entity, &GlobalTransform, &EntityType)>,
	players: Query<(Entity, &PlayerBody)>,
) {
	for event in server_events.read() {
		match event {
			ServerEvent::ClientConnected { client_id } => {
				info!("Client connected: {}", client_id);

				server_commands.send(ServerCommand::SpawnEntity(
					ServerEntity(commands.spawn_empty().id()),
					EntityType::Player(*client_id),
					Vec3::new(0.0, 2.0, 0.0),
				));

				for (entity, transform, entity_type) in entities.iter() {
					server.send_message(
						*client_id,
						ServerChannel::Commands,
						bincode::serialize(&ServerCommand::SpawnEntity(
							ServerEntity(entity),
							*entity_type,
							transform.translation(),
						))
						.unwrap(),
					);
				}
			}
			ServerEvent::ClientDisconnected { client_id, reason } => {
				info!("Client disconnected: {} because {}", client_id, reason);

				// let entity = players
				// 	.iter()
				// 	.filter(|(_, p)| p.0 == *client_id)
				// 	.next()
				// 	.unwrap()
				// 	.0;
				// server_commands.send(ServerCommand::DespawnEntity(entity));
			}
		}
	}
}

fn server_send(mut server: ResMut<RenetServer>, mut server_commands: EventReader<ServerCommand>) {
	for command in server_commands.read() {
		let command = bincode::serialize(&command).unwrap();
		server.broadcast_message(ServerChannel::Commands, command);
	}
}

fn server_send_component<ComponentType>(
	mut server: ResMut<RenetServer>,
	entities: Query<(Entity, &ComponentType)>,
) where
	ComponentType: Component + Clone + Serialize,
{
	for (entity, component) in entities.iter() {
		server.broadcast_message(
			ServerChannel::NetworkedEntities,
			bincode::serialize(&(entity, component.clone())).unwrap(),
		);
	}
}

fn server_send_component_entity_not_player<ComponentType>(
	mut server: ResMut<RenetServer>,
	entities: Query<(Entity, &ComponentType, &EntityType)>,
) where
	ComponentType: Component + Clone + Serialize,
{
	for (entity, component, entity_type) in entities.iter() {
		match entity_type {
			EntityType::Player(_) => continue,

			_ => {
				server.broadcast_message(
					ServerChannel::NetworkedEntities,
					bincode::serialize(&(entity, component.clone())).unwrap(),
				);
			}
		}
	}
}

fn server_recieve(mut server: ResMut<RenetServer>) {
	for client_id in server.clients_id() {
		while let Some(message) = server.receive_message(client_id, ClientChannel::Commands) {
			let server_message: Vec3 = bincode::deserialize(&message).unwrap();
			debug!("Recieved {}", server_message);
		}
	}
}
