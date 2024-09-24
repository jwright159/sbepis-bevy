use std::any::type_name;
use std::fmt::Debug;
use std::net::UdpSocket;
use std::time::SystemTime;

use bevy::prelude::*;
use bevy_renet::renet::transport::{
	ClientAuthentication, NetcodeClientTransport, NetcodeTransportError,
};
use bevy_renet::renet::{Bytes, RenetClient};
use bevy_renet::transport::NetcodeClientPlugin;
use bevy_renet::{client_connected, RenetClientPlugin};
use sbepis::entity::RandomInput;
use sbepis::netcode::*;
use serde::Deserialize;

fn main() {
	App::new()
		.add_plugins((
			sbepis::CommonPlugin::new("SBEPIS"),
			ClientPlugin,
			sbepis::player_controller::SpawnPlayerPlugin,
		))
		.add_systems(Startup, sbepis::hide_mouse)
		.run();
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Connected;

struct ClientPlugin;
impl Plugin for ClientPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(RenetClientPlugin);
		app.add_plugins(NetcodeClientPlugin);

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
		app.insert_resource(ServerState::Client {
			server_to_client: default(),
		});

		// If any error is found we just panic
		fn panic_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
			if let Some(e) = renet_error.read().next() {
				panic!("{}", e);
			}
		}

		app.add_systems(Update, panic_on_error_system);

		app.add_event::<NetworkedEntityEvent>();

		app.configure_sets(Update, Connected.run_if(client_connected));
		app.add_systems(
			Update,
			(
				client_send,
				client_recieve,
				client_recieve_component_events,
				client_recieve_component::<RandomInput>,
				client_recieve_component::<Transform>,
			)
				.in_set(Connected),
		);
	}
}

fn client_send(mut client: ResMut<RenetClient>) {
	let input_message = bincode::serialize(&Vec3::X).unwrap();
	client.send_message(ClientChannel::Commands, input_message);
}

fn client_recieve(
	mut commands: Commands,
	mut client: ResMut<RenetClient>,
	mut server_commands: EventWriter<ServerCommand>,
	mut entity_map: ResMut<ServerState>,
) {
	while let Some(message) = client.receive_message(ServerChannel::Commands) {
		let mut command: ServerCommand = bincode::deserialize(&message).unwrap();
		debug!("Recieved {:?}", command);
		match &mut command {
			ServerCommand::SpawnEntity(entity, _, _) => {
				let client_entity = commands.spawn_empty().id();
				entity.0 = entity_map.decode_and_insert_entity(*entity, client_entity);
			}
			ServerCommand::DespawnEntity(entity) => {
				if let Some(client_entity) = entity_map.decode_entity(*entity) {
					entity.0 = client_entity;
				} else {
					continue;
				}
			}
		}
		server_commands.send(command);
	}
}

fn client_recieve_component_events(
	mut client: ResMut<RenetClient>,
	mut events: EventWriter<NetworkedEntityEvent>,
) {
	while let Some(data) = client.receive_message(ServerChannel::NetworkedEntities) {
		events.send(NetworkedEntityEvent(data));
	}
}

fn client_recieve_component<T>(
	mut commands: Commands,
	mut events: EventReader<NetworkedEntityEvent>,
	entity_map: Res<ServerState>,
) where
	T: Component + for<'de> Deserialize<'de> + Debug,
{
	for NetworkedEntityEvent(data) in events.read() {
		if let Ok((entity, component)) = bincode::deserialize::<(ServerEntity, T)>(data) {
			if let Some(entity) = entity_map.decode_entity(entity) {
				debug!(
					"Recieved component {} for entity {}",
					type_name::<T>(),
					entity
				);
				commands.entity(entity).insert(component);
			}
		}
	}
}

#[derive(Event)]
pub struct NetworkedEntityEvent(Bytes);
