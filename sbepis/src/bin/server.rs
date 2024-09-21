use std::net::UdpSocket;
use std::time::SystemTime;

use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;
use bevy_rapier3d::prelude::*;
use bevy_renet::renet::transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_renet::renet::RenetServer;
use bevy_renet::transport::NetcodeServerPlugin;
use sbepis::entity::{Healing, RandomInput, RotateTowardMovement, SpawnHealthBar, TargetPlayer};
use sbepis::netcode::*;
use sbepis::{gridbox_material, main_bundles::*};

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

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
) {
	let green_material = gridbox_material("green1", &mut materials, &asset_server);
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
		Name::new("Consort"),
		EntityBundle::new(
			Transform::from_translation(Vec3::new(-5.0, 10.0, 0.0)),
			meshes.add(
				Capsule3d::new(0.25, 0.5)
					.mesh()
					.rings(1)
					.latitudes(8)
					.longitudes(16)
					.uv_profile(CapsuleUvProfile::Fixed),
			),
			gridbox_material("magenta", &mut materials, &asset_server),
			Collider::capsule_y(0.25, 0.25),
		),
		SpawnHealthBar,
		RandomInput::default(),
		Healing(0.2),
		RotateTowardMovement,
	));
	commands.spawn((
		Name::new("Imp"),
		EntityBundle::new(
			Transform::from_translation(Vec3::new(-6.0, 10.0, 0.0)),
			meshes.add(
				Capsule3d::new(0.25, 0.5)
					.mesh()
					.rings(1)
					.latitudes(8)
					.longitudes(16)
					.uv_profile(CapsuleUvProfile::Fixed),
			),
			gridbox_material("brown", &mut materials, &asset_server),
			Collider::capsule_y(0.25, 0.25),
		),
		SpawnHealthBar,
		TargetPlayer,
		RotateTowardMovement,
	));
}

struct ServerPlugin;
impl Plugin for ServerPlugin {
	fn build(&self, app: &mut App) {
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
