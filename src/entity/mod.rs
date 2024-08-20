use bevy::prelude::*;

use crate::gridbox_material;

pub use self::health::Health;
use self::health::*;
pub use self::movement::MovementInput;
use self::movement::*;
pub use self::orientation::GravityOrientation;
use self::orientation::*;

pub mod health;
pub mod movement;
pub mod orientation;

pub struct EntityPlugin;
impl Plugin for EntityPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(PostStartup, spawn_health_bars).add_systems(
			Update,
			(
				orient,
				strafe,
				kill_entities_with_no_health,
				update_health_bars_health,
				update_health_bars_size,
			),
		);
	}
}

fn spawn_health_bars(
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
					max_health: 3.,
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

fn update_health_bars_health(mut health_bars: Query<&mut HealthBar>, healths: Query<&Health>) {
	for mut health_bar in health_bars.iter_mut() {
		let health = healths
			.get(health_bar.entity)
			.expect("HealthBar entity has no Health component");
		health_bar.health = health.0;
	}
}

fn update_health_bars_size(mut health_bars: Query<(&HealthBar, &mut Transform)>) {
	for (health_bar, mut transform) in health_bars.iter_mut() {
		transform.translation.x = transform.scale.x * 0.5 - 0.5;
		transform.scale = Vec3::new(health_bar.health / health_bar.max_health, 1.0, 1.0);
	}
}
