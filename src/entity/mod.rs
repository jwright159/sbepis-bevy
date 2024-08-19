use bevy::prelude::*;

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
		app.add_systems(Update, (orient, strafe, kill_entities_with_no_health));
	}
}
