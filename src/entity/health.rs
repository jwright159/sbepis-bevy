use bevy::prelude::*;

use crate::gridbox_material;

#[derive(Component)]
pub struct GelViscosity(pub f32);

#[derive(Component)]
pub struct CanDealDamage;

#[derive(Component)]
pub struct SpawnHealthBar;

#[derive(Component)]
pub struct HealthBar {
	pub entity: Entity,
	pub health: f32,
	pub max_health: f32,
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

		let size = Vec3::new(1.0, 0.3, 0.3);
		let outline = 0.05;
		commands
			.spawn((
				Name::new("Health Bar"),
				HealthBar {
					entity,
					health: 0.,
					max_health: 5.,
				},
				PbrBundle {
					transform: Transform::from_translation(Vec3::Y * 1.5),
					mesh: meshes.add(Cuboid::from_size(size)),
					material: gridbox_material("red", &mut materials, &asset_server),
					..default()
				},
			))
			.set_parent(entity);

		commands
			.spawn((
				Name::new("Health Bar Outline"),
				PbrBundle {
					transform: Transform::from_translation(Vec3::Y * 1.5).with_scale(Vec3::NEG_ONE),
					mesh: meshes.add(Cuboid::from_size(size + Vec3::splat(outline))),
					material: gridbox_material("grey1", &mut materials, &asset_server),
					..default()
				},
			))
			.set_parent(entity);
	}
}

pub fn update_health_bars_health(
	mut commands: Commands,
	mut health_bars: Query<&mut HealthBar>,
	healths: Query<&GelViscosity>,
) {
	for mut health_bar in health_bars.iter_mut() {
		let health = match healths.get(health_bar.entity) {
			Ok(health) => health,
			Err(_) => {
				commands.entity(health_bar.entity).despawn_recursive();
				continue;
			}
		};
		health_bar.health = health.0;
	}
}

pub fn update_health_bars_size(mut health_bars: Query<(&HealthBar, &mut Transform)>) {
	for (health_bar, mut transform) in health_bars.iter_mut() {
		transform.translation.x = transform.scale.x * 0.5 - 0.5;
		transform.scale = Vec3::new(health_bar.health / health_bar.max_health, 1.0, 1.0);
	}
}
