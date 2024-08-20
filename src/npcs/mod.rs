use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;
use bevy_rapier3d::geometry::Collider;

use crate::entity::{Healing, RandomInput, RotateTowardMovement, SpawnHealthBar, TargetPlayer};
use crate::gridbox_material;
use crate::main_bundles::EntityBundle;

pub struct NpcPlugin;
impl Plugin for NpcPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(Startup, setup);
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
