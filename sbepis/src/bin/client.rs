use std::net::UdpSocket;
use std::time::SystemTime;

use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use bevy_renet::renet::transport::{
	ClientAuthentication, NetcodeClientTransport, NetcodeTransportError,
};
use bevy_renet::renet::RenetClient;
use bevy_renet::transport::NetcodeClientPlugin;
use bevy_renet::{client_connected, RenetClientPlugin};
use sbepis::netcode::*;

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

#[derive(Debug, Resource)]
struct CurrentClientId(u64);

#[derive(Debug, Default, Resource)]
struct ServerClientEntityMap {
	server_to_client: EntityHashMap<Entity>,
	// client_to_server: EntityHashMap<Entity>,
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
		app.insert_resource(ServerClientEntityMap::default());

		// If any error is found we just panic
		#[allow(clippy::never_loop)]
		fn panic_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
			for e in renet_error.read() {
				panic!("{}", e);
			}
		}

		app.add_systems(Update, panic_on_error_system);

		app.configure_sets(Update, Connected.run_if(client_connected));
		app.add_systems(Update, (client_send, client_recieve).in_set(Connected));
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
	mut entity_map: ResMut<ServerClientEntityMap>,
) {
	while let Some(message) = client.receive_message(ServerChannel::Commands) {
		let mut command: ServerCommand = bincode::deserialize(&message).unwrap();
		println!("Recieved {:?}", command);
		match &mut command {
			ServerCommand::SpawnEntity(entity, _, _) => {
				let client_entity = commands.spawn_empty().id();
				entity_map.server_to_client.insert(*entity, client_entity);
				*entity = client_entity;
			}
			ServerCommand::DespawnEntity(entity) => {
				let client_entity = *entity_map.server_to_client.get(entity).unwrap();
				*entity = client_entity;
			}
		}
		server_commands.send(command);
	}
}
