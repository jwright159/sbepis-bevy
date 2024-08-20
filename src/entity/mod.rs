use bevy::prelude::*;

use self::health::*;
pub use self::health::{GelViscosity, Healing, SpawnHealthBar};
use self::movement::*;
pub use self::movement::{MovementInput, RandomInput, RotateTowardMovement, TargetPlayer};
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
				random_vec2,
				target_player,
				strafe,
				rotate_toward_movement,
				despawn_invalid_health_bars,
				update_health_bars_health,
				update_health_bars_size,
				heal,
			),
		);
	}
}
