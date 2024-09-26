use std::any::TypeId;
use std::time::Duration;

use bevy::prelude::*;
use bevy::utils::hashbrown::HashMap;
use bevy::utils::EntityHash;
use bevy_rapier3d::prelude::Velocity;
use bevy_renet::renet::{Bytes, ChannelConfig, ClientId, ConnectionConfig, SendType};
use serde::{Deserialize, Serialize};

use crate::{ConsortBundle, CubeBundle, ImpBundle, PlayerBundle};

pub const PROTOCOL_ID: u64 = 0xdeadbeef;

pub fn connection_config() -> ConnectionConfig {
	ConnectionConfig {
		available_bytes_per_tick: 1024 * 1024,
		client_channels_config: ClientChannel::channels_config(),
		server_channels_config: ServerChannel::channels_config(),
	}
}

pub enum ClientChannel {
	Input,
	Commands,
}

impl From<ClientChannel> for u8 {
	fn from(channel_id: ClientChannel) -> Self {
		match channel_id {
			ClientChannel::Commands => 0,
			ClientChannel::Input => 1,
		}
	}
}

impl ClientChannel {
	pub fn channels_config() -> Vec<ChannelConfig> {
		vec![
			ChannelConfig {
				channel_id: Self::Input.into(),
				max_memory_usage_bytes: 5 * 1024 * 1024,
				send_type: SendType::ReliableOrdered {
					resend_time: Duration::ZERO,
				},
			},
			ChannelConfig {
				channel_id: Self::Commands.into(),
				max_memory_usage_bytes: 5 * 1024 * 1024,
				send_type: SendType::ReliableOrdered {
					resend_time: Duration::ZERO,
				},
			},
		]
	}
}

pub enum ServerChannel {
	Commands,
	NetworkedEntities,
}

impl From<ServerChannel> for u8 {
	fn from(channel_id: ServerChannel) -> Self {
		match channel_id {
			ServerChannel::NetworkedEntities => 0,
			ServerChannel::Commands => 1,
		}
	}
}

impl ServerChannel {
	pub fn channels_config() -> Vec<ChannelConfig> {
		vec![
			ChannelConfig {
				channel_id: Self::NetworkedEntities.into(),
				max_memory_usage_bytes: 10 * 1024 * 1024,
				send_type: SendType::Unreliable,
			},
			ChannelConfig {
				channel_id: Self::Commands.into(),
				max_memory_usage_bytes: 10 * 1024 * 1024,
				send_type: SendType::ReliableOrdered {
					resend_time: Duration::from_millis(200),
				},
			},
		]
	}
}

#[derive(Debug, Serialize, Deserialize, Event)]
pub enum ServerCommand {
	SpawnEntity(ServerEntity, EntityType, Vec3),
	DespawnEntity(ServerEntity),
}

#[derive(Debug, Serialize, Deserialize, Component, Clone, Copy)]
pub enum EntityType {
	Cube,
	Imp,
	Consort,
	Player(ClientId),
}

pub fn server_commands(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
	mut server_commands: EventReader<ServerCommand>,
	client_id: Option<Res<CurrentClientId>>,
	mut server_state: ResMut<ServerState>,
) {
	for command in server_commands.read() {
		match command {
			ServerCommand::SpawnEntity(entity, entity_type, position) => {
				if let Some(entity) = server_state.decode_entity(*entity) {
					match entity_type {
						EntityType::Cube => {
							commands.entity(entity).insert(CubeBundle::new(
								*position,
								&mut meshes,
								&mut materials,
								&asset_server,
							));
						}
						EntityType::Consort => {
							commands.entity(entity).insert(ConsortBundle::new(
								*position,
								&mut meshes,
								&mut materials,
								&asset_server,
							));
						}
						EntityType::Imp => {
							commands.entity(entity).insert(ImpBundle::new(
								*position,
								&mut meshes,
								&mut materials,
								&asset_server,
							));
						}
						EntityType::Player(player_id) => {
							commands.entity(entity).insert(PlayerBundle::new(
								*position,
								&mut meshes,
								&mut materials,
								&asset_server,
							));

							if let Some(ref client_id) = client_id {
								if client_id.0 == player_id.raw() {
									commands.entity(entity).insert(ClientPlayer);
								}
							}
						}
					}
				}
			}
			ServerCommand::DespawnEntity(entity) => {
				if let Some(entity) = server_state.decode_and_remove_entity(*entity) {
					commands.entity(entity).despawn();
				}
			}
		}
	}
}

#[derive(Component)]
pub struct ClientPlayer;

#[derive(Debug, Resource)]
#[allow(dead_code)]
pub struct CurrentClientId(pub u64);

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ServerEntity(pub Entity);

#[derive(Resource, Debug)]
pub enum ServerState {
	Server,
	Client {
		server_to_client_entities: HashMap<ServerEntity, Entity, EntityHash>,
		client_to_server_entities: HashMap<Entity, ServerEntity, EntityHash>,
		server_to_client_types: HashMap<u64, u64>,
		client_to_server_types: HashMap<u64, u64>,
	},
}

impl ServerState {
	pub fn decode_entity(&self, entity: ServerEntity) -> Option<Entity> {
		match self {
			Self::Server => Some(entity.0),
			Self::Client {
				server_to_client_entities: server_to_client,
				client_to_server_entities: _,
				server_to_client_types: _,
				client_to_server_types: _,
			} => server_to_client.get(&entity).copied(),
		}
	}

	pub fn decode_and_insert_entity(
		&mut self,
		entity: ServerEntity,
		client_entity: Entity,
	) -> Entity {
		match self {
			Self::Server => entity.0,
			Self::Client {
				server_to_client_entities: server_to_client,
				client_to_server_entities: client_to_server,
				server_to_client_types: _,
				client_to_server_types: _,
			} => {
				server_to_client.insert(entity, client_entity);
				client_to_server.insert(client_entity, entity);
				entity.0
			}
		}
	}

	pub fn decode_and_remove_entity(&mut self, entity: ServerEntity) -> Option<Entity> {
		match self {
			Self::Server => Some(entity.0),
			Self::Client {
				server_to_client_entities: server_to_client,
				client_to_server_entities: client_to_server,
				server_to_client_types: _,
				client_to_server_types: _,
			} => {
				let client_entity = server_to_client.remove(&entity);
				if let Some(client_entity) = client_entity {
					client_to_server.remove(&client_entity);
				}
				client_entity
			}
		}
	}

	pub fn encode_entity(&self, entity: Entity) -> Option<ServerEntity> {
		match self {
			Self::Server => Some(ServerEntity(entity)),
			Self::Client {
				server_to_client_entities: _,
				client_to_server_entities: client_to_server,
				server_to_client_types: _,
				client_to_server_types: _,
			} => client_to_server.get(&entity).copied(),
		}
	}

	pub fn insert_server_types(&mut self, types: SyncServerTypes) {
		if let Self::Client {
			server_to_client_entities: _,
			client_to_server_entities: _,
			server_to_client_types,
			client_to_server_types,
		} = self
		{
			let client_types = SyncServerTypes::default();
			server_to_client_types.insert(types.transform, client_types.transform);
			client_to_server_types.insert(client_types.transform, types.transform);
			server_to_client_types.insert(types.velocity, client_types.velocity);
			client_to_server_types.insert(client_types.velocity, types.velocity);
		}
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncServerTypes {
	pub transform: u64,
	pub velocity: u64,
}

impl Default for SyncServerTypes {
	fn default() -> Self {
		Self {
			transform: hash_type_id(&TypeId::of::<Transform>()),
			velocity: hash_type_id(&TypeId::of::<Velocity>()),
		}
	}
}

fn hash_type_id(type_id: &TypeId) -> u64 {
	use std::collections::hash_map::DefaultHasher;
	use std::hash::{Hash, Hasher};

	let mut hasher = DefaultHasher::new();
	type_id.hash(&mut hasher);
	hasher.finish()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentData {
	pub entity: ServerEntity,
	pub component_type: u64,
	#[serde(
		serialize_with = "serialize_bytes",
		deserialize_with = "deserialize_bytes"
	)]
	pub data: Bytes,
}

fn serialize_bytes<S>(data: &Bytes, serializer: S) -> Result<S::Ok, S::Error>
where
	S: serde::Serializer,
{
	serializer.serialize_bytes(data.as_ref())
}

fn deserialize_bytes<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
where
	D: serde::Deserializer<'de>,
{
	let data = <&[u8]>::deserialize(deserializer)?;
	Ok(Bytes::copy_from_slice(data))
}
