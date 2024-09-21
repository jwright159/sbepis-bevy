use std::net::UdpSocket;
use std::time::SystemTime;

use bevy::prelude::*;
use bevy_renet::client_connected;
use bevy_renet::renet::transport::{
	ClientAuthentication, NetcodeClientTransport, NetcodeTransportError,
};
use bevy_renet::renet::RenetClient;
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

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Connected;

struct ClientPlugin;
impl Plugin for ClientPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(bevy_renet::transport::NetcodeClientPlugin);

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

		app.configure_sets(Update, Connected.run_if(client_connected));
		app.add_systems(Update, (client_send, client_recieve).in_set(Connected));
	}
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
