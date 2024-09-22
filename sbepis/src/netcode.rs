use std::time::Duration;

use bevy::prelude::*;
use bevy_renet::renet::{ChannelConfig, ClientId, ConnectionConfig, SendType};
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
	SpawnEntity(Entity, EntityType, Vec3),
	DespawnEntity(Entity),
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
) {
	for command in server_commands.read() {
		match command {
			ServerCommand::SpawnEntity(entity, entity_type, position) => match entity_type {
				EntityType::Cube => {
					commands.entity(*entity).insert(CubeBundle::new(
						*position,
						&mut meshes,
						&mut materials,
						&asset_server,
					));
				}
				EntityType::Consort => {
					commands.entity(*entity).insert(ConsortBundle::new(
						*position,
						&mut meshes,
						&mut materials,
						&asset_server,
					));
				}
				EntityType::Imp => {
					commands.entity(*entity).insert(ImpBundle::new(
						*position,
						&mut meshes,
						&mut materials,
						&asset_server,
					));
				}
				EntityType::Player(player_id) => {
					commands.entity(*entity).insert(PlayerBundle::new(
						*position,
						&mut meshes,
						&mut materials,
						&asset_server,
					));

					if let Some(ref client_id) = client_id {
						if client_id.0 == player_id.raw() {
							commands.entity(*entity).insert(ClientPlayer);
						}
					}
				}
			},
			ServerCommand::DespawnEntity(entity) => {
				commands.entity(*entity).despawn();
			}
		}
	}
}

#[derive(Component)]
pub struct ClientPlayer;

#[derive(Debug, Resource)]
#[allow(dead_code)]
pub struct CurrentClientId(pub u64);
