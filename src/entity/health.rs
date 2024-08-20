use bevy::prelude::*;

use crate::player_controller::PlayerCamera;
use crate::{gridbox_material, gridbox_material_extra, util::MapRange};

#[derive(Component)]
pub struct GelViscosity {
	pub value: f32,
	pub max: f32,
}

#[derive(Component)]
pub struct CanDealDamage;

#[derive(Component)]
pub struct SpawnHealthBar;

#[derive(Component)]
pub struct GelVial {
	pub entity: Entity,
	pub health: f32,
	pub max_health: f32,
	pub root: Entity,
	pub glass: Entity,
	pub length: f32,
	pub height: f32,
}

pub fn spawn_health_bars(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
	entities: Query<Entity, With<SpawnHealthBar>>,
) {
	for entity in entities.iter() {
		commands.entity(entity).remove::<SpawnHealthBar>();

		let length = 1.0;
		let size = Vec3::new(length, 0.3, 0.3);
		let glass_outline = 0.01;
		let outline = 0.05;
		let height = 1.5;

		let root = commands
			.spawn((TransformBundle::default(), VisibilityBundle::default()))
			.id();

		commands
			.spawn((
				Name::new("Gel Vial Outline"),
				PbrBundle {
					transform: Transform::from_scale(Vec3::NEG_ONE),
					mesh: meshes.add(Cuboid::from_size(size + Vec3::splat(outline))),
					material: gridbox_material("grey1", &mut materials, &asset_server),
					..default()
				},
			))
			.set_parent(root);

		let glass = commands
			.spawn((
				Name::new("Gel Vial Glass"),
				PbrBundle {
					mesh: meshes.add(Cuboid::from_size(size + Vec3::splat(glass_outline))),
					material: gridbox_material_extra(
						"clear",
						&mut materials,
						&asset_server,
						StandardMaterial {
							alpha_mode: AlphaMode::Blend,
							clearcoat: 1.0,
							clearcoat_perceptual_roughness: 0.0,
							..default()
						},
					),
					..default()
				},
			))
			.set_parent(root)
			.id();

		commands
			.spawn((
				Name::new("Gel Vial"),
				GelVial {
					entity,
					health: 0.,
					max_health: 5.,
					root,
					glass,
					length,
					height,
				},
				PbrBundle {
					mesh: meshes.add(Cuboid::from_size(size)),
					material: gridbox_material("red", &mut materials, &asset_server),
					..default()
				},
			))
			.set_parent(root);
	}
}

pub fn update_health_bars_health(
	mut commands: Commands,
	mut health_bars: Query<&mut GelVial>,
	healths: Query<&GelViscosity>,
) {
	for mut health_bar in health_bars.iter_mut() {
		let health = match healths.get(health_bar.entity) {
			Ok(health) => health,
			Err(_) => {
				commands.entity(health_bar.root).despawn_recursive();
				continue;
			}
		};
		health_bar.health = health.value;
		health_bar.max_health = health.max;
	}
}

pub fn update_health_bars_size(
	mut health_bars: Query<(&GelVial, &mut Transform), Without<PlayerCamera>>,
	mut transforms: Query<&mut Transform, (Without<GelVial>, Without<PlayerCamera>)>,
	player_camera: Query<&GlobalTransform, (Without<GelVial>, With<PlayerCamera>)>,
) {
	for (health_bar, mut transform) in health_bars.iter_mut() {
		let percentage = (health_bar.health / health_bar.max_health).max(0.0);
		transform.translation.x = percentage.map_range(0.0..1.0, (health_bar.length * 0.5)..0.0);
		transform.scale = Vec3::new(percentage, 1.0, 1.0);

		let [mut glass_transform, mut root_transform, entity_transform] =
			match transforms.get_many_mut([health_bar.glass, health_bar.root, health_bar.entity]) {
				Ok(transforms) => transforms,
				Err(_) => {
					continue; // Should be handled by update_health_bars_health
				}
			};
		let player_camera_transform = player_camera.get_single().expect("Player camera not found");

		glass_transform.translation.x = percentage.map_range(0.0..1.0, health_bar.length..0.0);

		root_transform.translation = entity_transform.transform_point(Vec3::Y * health_bar.height);
		root_transform.look_at(
			player_camera_transform.translation(),
			player_camera_transform.up(),
		);
	}
}

#[derive(Component)]
pub struct Healing(pub f32);

pub fn heal(mut healings: Query<(&Healing, &mut GelViscosity)>, time: Res<Time>) {
	for (healing, mut health) in healings.iter_mut() {
		health.value += healing.0 * time.delta_seconds();
		health.value = health.value.min(health.max);
	}
}
