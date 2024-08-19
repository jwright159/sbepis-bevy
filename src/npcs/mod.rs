use std::time::Duration;

use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;
use bevy::utils::HashMap;
use bevy_rapier3d::dynamics::LockedAxes;
use bevy_rapier3d::geometry::Collider;

use crate::gravity::GravityRigidbodyBundle;
use crate::gridbox_material;
use crate::player_controller::{strafe, GravityOrientation, Health};

pub struct NpcPlugin;
impl Plugin for NpcPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, setup);
		app.add_systems(Update, random_vec2.pipe(strafe::<Npc>));
	}
}

#[derive(Component)]
pub struct Npc;

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
) {
	commands.spawn((
		Name::new("Consort"),
		PbrBundle {
			transform: Transform::from_translation(Vec3::new(-5.0, 10.0, 0.0)),
			mesh: meshes.add(
				Capsule3d::new(0.25, 0.5)
					.mesh()
					.rings(1)
					.latitudes(8)
					.longitudes(16)
					.uv_profile(CapsuleUvProfile::Fixed),
			),
			material: gridbox_material("magenta", &mut materials, &asset_server),
			..default()
		},
		GravityRigidbodyBundle::default(),
		Collider::capsule_y(0.25, 0.25),
		GravityOrientation,
		Npc,
		LockedAxes::ROTATION_LOCKED,
		RandomInput::default(),
		Health(3.0),
	));
}

#[derive(Component, Default)]
pub struct RandomInput {
	pub input: Vec2,
	pub time_since_last_change: Duration,
	pub time_to_change: Duration,
}

pub fn random_vec2(
	mut input: Query<(Entity, &mut RandomInput)>,
	time: Res<Time>,
) -> HashMap<Entity, Vec2> {
	let mut map = HashMap::default();

	for (entity, mut random_input) in input.iter_mut() {
		random_input.time_since_last_change += time.delta();

		if random_input.time_since_last_change >= random_input.time_to_change {
			let angle = rand::random::<f32>() * std::f32::consts::TAU;
			let mag = rand::random::<f32>() + 0.2;
			random_input.input = Vec2::new(angle.cos(), angle.sin()) * mag;
			random_input.time_since_last_change = Duration::default();
			random_input.time_to_change =
				Duration::from_secs_f32(rand::random::<f32>() * 2.0 + 1.0);
		}

		map.insert(entity, random_input.input);
	}
	map
}
