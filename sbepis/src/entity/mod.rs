use bevy::prelude::*;

use self::health::*;
pub use self::health::{GelViscosity, Healing, SpawnHealthBar};
use self::movement::*;
pub use self::movement::{Movement, RandomInput, RotateTowardMovement, TargetPlayer};
pub use self::orientation::GravityOrientation;
use self::orientation::*;

pub mod health;
pub mod movement;
pub mod orientation;
pub mod spawner;

pub struct EntityPlugin;
impl Plugin for EntityPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<EntityKilled>().add_systems(
			Update,
			(
				orient,
				random_vec2,
				target_player,
				strafe,
				rotate_toward_movement,
				spawn_health_bars,
				despawn_invalid_health_bars,
				update_health_bars_health,
				update_health_bars_size,
				heal,
				kill_entities,
			),
		);
	}
}

#[derive(Event)]
pub struct EntityKilled {
	pub entity: Entity,
}

fn kill_entities(mut ev_killed: EventReader<EntityKilled>, mut commands: Commands) {
	for ev in ev_killed.read() {
		commands.entity(ev.entity).despawn_recursive();
	}
}
