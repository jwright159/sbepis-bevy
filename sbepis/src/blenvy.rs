use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::gravity::{GravityPoint, GravityPriority};
use crate::{ok_or_continue, some_or_continue};

pub struct BlenvyPlugin;

impl Plugin for BlenvyPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(::blenvy::BlenvyPlugin::default());

		app.register_type::<MeshColliderBlundle>()
			.register_type::<PlanetBlundle>();

		app.add_systems(PreUpdate, (create_mesh_collider, create_planet));
	}
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MeshColliderBlundle;

pub fn create_mesh_collider(
	scenes: Query<Entity, With<MeshColliderBlundle>>,
	children: Query<&Children>,
	meshes: Query<&Mesh3d>,
	mesh_assets: Res<Assets<Mesh>>,
	mut commands: Commands,
) {
	for scene in scenes.iter() {
		let mut num_colliders = 0;

		for child in children.iter_descendants(scene) {
			let mesh = ok_or_continue!(meshes.get(child));
			let mesh = some_or_continue!(mesh_assets.get(&mesh.0));
			let collider = Collider::from_bevy_mesh(mesh, &ComputedColliderShape::default())
				.expect("Couldn't make a mesh collider");
			commands.entity(child).insert(collider);
			num_colliders += 1;
		}

		if num_colliders > 0 {
			commands.entity(scene).remove::<MeshColliderBlundle>();
		}
	}
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlanetBlundle {
	pub radius: f32,
	pub gravity: f32,
}

pub fn create_planet(scenes: Query<(Entity, &PlanetBlundle)>, mut commands: Commands) {
	for (scene, planet) in scenes.iter() {
		commands.entity(scene).remove::<PlanetBlundle>().insert((
			RigidBody::Fixed,
			GravityPoint {
				standard_radius: planet.radius,
				acceleration_at_radius: planet.gravity,
			},
			GravityPriority(0),
		));
	}
}
