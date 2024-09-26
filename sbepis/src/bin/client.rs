use std::fmt::Debug;
use std::net::UdpSocket;
use std::time::SystemTime;

use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use bevy_renet::renet::transport::{
	ClientAuthentication, NetcodeClientTransport, NetcodeTransportError,
};
use bevy_renet::renet::{Bytes, RenetClient};
use bevy_renet::transport::NetcodeClientPlugin;
use bevy_renet::{client_connected, RenetClientPlugin};
use sbepis::entity::RandomInput;
use sbepis::netcode::*;
use serde::{Deserialize, Serialize};

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
			server_to_client_entities: default(),
			client_to_server_entities: default(),
			server_to_client_types: default(),
			client_to_server_types: default(),
		});

		// If any error is found we just panic
		fn panic_on_error_system(mut renet_error: EventReader<NetcodeTransportError>) {
			if let Some(e) = renet_error.read().next() {
				panic!("{}", e);
			}
		}

		app.add_systems(Update, panic_on_error_system);

		app.configure_sets(Update, Connected.run_if(client_connected));
		app.add_systems(
			Update,
			(
				client_send_commands,
				client_send_component_player::<Transform>,
				client_send_component_player::<Velocity>,
				client_recieve_commands,
				client_recieve_component_events,
				client_recieve_component::<RandomInput>,
				client_recieve_component::<Transform>,
				client_recieve_component::<Velocity>,
			)
				.in_set(Connected),
		);
	}
}

fn client_send_commands(mut client: ResMut<RenetClient>) {}

fn client_recieve_commands(
	mut commands: Commands,
	mut client: ResMut<RenetClient>,
	mut server_commands: EventWriter<ServerCommand>,
	mut server_state: ResMut<ServerState>,
) {
	while let Some(message) = client.receive_message(ServerChannel::Commands) {
		let mut command: ServerCommand = bincode::deserialize(&message).unwrap();
		debug!("Recieved {:?}", command);
		match &mut command {
			ServerCommand::SpawnEntity(entity, _, _) => {
				let client_entity = commands.spawn_empty().id();
				entity.0 = server_state.decode_and_insert_entity(*entity, client_entity);
			}
			ServerCommand::DespawnEntity(entity) => {
				if let Some(client_entity) = server_state.decode_entity(*entity) {
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
	mut commands: Commands,
	mut client: ResMut<RenetClient>,
	server_state: Res<ServerState>,
) {
	while let Some(data) = client.receive_message(ServerChannel::NetworkedEntities) {
		if let Ok((entity, component)) = bincode::deserialize::<(ServerEntity, ComponentType)>(data)
		{
			if let Some(entity) = server_state.decode_entity(entity) {
				commands.entity(entity).insert(component);
			}
		}
	}
}

#[derive(Debug)]
struct SomeObject(i32);

fn foo() {
	let foo = SomeObject(5);
	bar(foo);
	println!("foo: {:?}", foo);
}

fn bar(num: SomeObject) {
	println!("num: {:?}", num);
}

fn client_send_component_player<ComponentType>(
	mut client: ResMut<RenetClient>,
	entities: Query<(Entity, &ComponentType), With<ClientPlayer>>,
	server_state: Res<ServerState>,
) where
	ComponentType: Component + Clone + Serialize + Debug,
{
	for (entity, component) in entities.iter() {
		if let Some(entity) = server_state.encode_entity(entity) {
			debug!("Sent {:?} {:?}", entity, component);
			let data = bincode::serialize(&(entity, component.clone())).unwrap();
			client.send_message(ClientChannel::Input, data);
		}
	}
}
