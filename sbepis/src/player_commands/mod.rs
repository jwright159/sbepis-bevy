mod commands;
mod note_holder;
mod notes;
mod staff;

use bevy_butler::*;

use crate::menus::InputManagerMenuPlugin;

use self::commands::*;
use self::notes::*;
use self::staff::*;

#[butler_plugin(build(
	add_plugins(InputManagerMenuPlugin::<CloseStaffAction>::default()),
	add_plugins(InputManagerMenuPlugin::<PlayNoteAction>::default()),
))]
pub struct PlayerCommandsPlugin;
