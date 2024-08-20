use bevy::prelude::*;

use self::health::*;
pub use self::health::{GelViscosity, Healing};
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
				update_health_bars_health,
				update_health_bars_size,
				heal,
			),
		);
	}
}
