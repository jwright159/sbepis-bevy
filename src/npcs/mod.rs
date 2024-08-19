use std::time::Duration;

use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;
use bevy_rapier3d::geometry::Collider;

use crate::gridbox_material;
use crate::main_bundles::EntityBundle;
use crate::player_controller::{MovementInput, PlayerBody};

pub struct NpcPlugin;
impl Plugin for NpcPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, setup);
		app.add_systems(Update, random_vec2);
		app.add_systems(Update, target_player);
	}
}

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
) {
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
		RandomInput::default(),
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
		TargetPlayer,
	));
}

#[derive(Component, Default)]
pub struct RandomInput {
	pub input: Vec2,
	pub time_since_last_change: Duration,
	pub time_to_change: Duration,
}

pub fn random_vec2(mut input: Query<(&mut RandomInput, &mut MovementInput)>, time: Res<Time>) {
	for (mut random_input, mut movement_input) in input.iter_mut() {
		random_input.time_since_last_change += time.delta();

		if random_input.time_since_last_change >= random_input.time_to_change {
			let angle = rand::random::<f32>() * std::f32::consts::TAU;
			let mag = rand::random::<f32>() + 0.2;
			random_input.input = Vec2::new(angle.cos(), angle.sin()) * mag;
			random_input.time_since_last_change = Duration::default();
			random_input.time_to_change =
				Duration::from_secs_f32(rand::random::<f32>() * 2.0 + 1.0);
		}

		movement_input.0 = random_input.input;
	}
}

#[derive(Component)]
pub struct TargetPlayer;

pub fn target_player(
	mut target_players: Query<(&Transform, &mut MovementInput), With<TargetPlayer>>,
	player: Query<&Transform, With<PlayerBody>>,
) {
	let player_transform = player.single();
	for (transform, mut input) in target_players.iter_mut() {
		let direction = player_transform.translation - transform.translation;
		let direction_local = transform.rotation.inverse() * direction;
		let input_direction = direction_local.xz().normalize();
		input.0 = input_direction;
	}
}
