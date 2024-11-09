mod commands;
mod note_holder;
mod notes;
mod staff;

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::input::button_event;
use crate::input::spawn_input_manager;
use crate::input::{action_event, input_manager_bundle};
use crate::menus::{InputManagerMenuPlugin, Menu, MenuWithInputManager, MenuWithoutMouse};

use self::commands::*;
use self::note_holder::*;
use self::notes::*;
use self::staff::*;

pub struct PlayerCommandsPlugin;

impl Plugin for PlayerCommandsPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(InputManagerPlugin::<ToggleStaffAction>::default())
			.add_plugins(InputManagerMenuPlugin::<PlayNoteAction>::default())
			.add_event::<NotePlayedEvent>()
			.add_event::<CommandSentEvent>()
			.add_event::<ClearNotesEvent>()
			.add_event::<ToggleStaffEvent>()
			.add_event::<PingCommandEvent>()
			.add_event::<KillCommandEvent>()
			.init_resource::<NotePatternPlayer>()
			.init_resource::<StaffState>()
			.add_systems(
				Startup,
				(
					spawn_staff,
					spawn_staff_menu,
					spawn_input_manager(
						InputMap::default()
							.with(ToggleStaffAction::ToggleStaff, KeyCode::Backquote),
						true,
					),
				),
			)
			.add_systems(
				PreUpdate,
				(
					action_event(NotePlayedEvent::from_play_note_action),
					button_event(ToggleStaffAction::ToggleStaff, ToggleStaffEvent::default),
				),
			)
			.add_systems(
				Update,
				(
					(
						toggle_staff,
						(show_staff, enable_note_input).run_if(is_staff_open),
						(hide_staff, disable_note_input, clear_notes).run_if(not(is_staff_open)),
					)
						.chain()
						.run_if(on_event::<ToggleStaffEvent>()),
					(spawn_note_audio, add_note_to_holder, add_note_to_player)
						.run_if(on_event::<NotePlayedEvent>()),
					(
						check_note_patterns::<PingCommandEvent>,
						check_note_patterns::<KillCommandEvent>,
					),
					(
						clear_notes.run_if(on_event::<CommandSentEvent>()),
						ping.run_if(on_event::<PingCommandEvent>()),
						kill.run_if(on_event::<KillCommandEvent>()),
					),
					(clear_holder_notes, clear_player_notes).run_if(on_event::<ClearNotesEvent>()),
				)
					.chain(),
			);
	}
}

fn spawn_staff_menu(mut commands: Commands) {
	commands.spawn((
		input_manager_bundle(
			InputMap::default()
				.with(PlayNoteAction::C4, KeyCode::KeyZ)
				.with(PlayNoteAction::CS4, KeyCode::KeyS)
				.with(PlayNoteAction::D4, KeyCode::KeyX)
				.with(PlayNoteAction::DS4, KeyCode::KeyD)
				.with(PlayNoteAction::E4, KeyCode::KeyC)
				.with(PlayNoteAction::F4, KeyCode::KeyV)
				.with(PlayNoteAction::FS4, KeyCode::KeyG)
				.with(PlayNoteAction::G4, KeyCode::KeyB)
				.with(PlayNoteAction::GS4, KeyCode::KeyH)
				.with(PlayNoteAction::A4, KeyCode::KeyN)
				.with(PlayNoteAction::AS4, KeyCode::KeyJ)
				.with(PlayNoteAction::B4, KeyCode::KeyM)
				.with(PlayNoteAction::C5, KeyCode::Comma)
				.with(PlayNoteAction::CS5, KeyCode::KeyL)
				.with(PlayNoteAction::D5, KeyCode::Period)
				.with(PlayNoteAction::DS5, KeyCode::Semicolon)
				.with(PlayNoteAction::E5, KeyCode::Slash)
				.with(PlayNoteAction::C5, KeyCode::KeyQ)
				.with(PlayNoteAction::CS5, KeyCode::Digit2)
				.with(PlayNoteAction::D5, KeyCode::KeyW)
				.with(PlayNoteAction::DS5, KeyCode::Digit3)
				.with(PlayNoteAction::E5, KeyCode::KeyE)
				.with(PlayNoteAction::F5, KeyCode::KeyR)
				.with(PlayNoteAction::FS5, KeyCode::Digit5)
				.with(PlayNoteAction::G5, KeyCode::KeyT)
				.with(PlayNoteAction::GS5, KeyCode::Digit6)
				.with(PlayNoteAction::A5, KeyCode::KeyY)
				.with(PlayNoteAction::AS5, KeyCode::Digit7)
				.with(PlayNoteAction::B5, KeyCode::KeyU)
				.with(PlayNoteAction::C6, KeyCode::KeyI)
				.with(PlayNoteAction::CS6, KeyCode::Digit9)
				.with(PlayNoteAction::D6, KeyCode::KeyO)
				.with(PlayNoteAction::DS6, KeyCode::Digit0)
				.with(PlayNoteAction::E6, KeyCode::KeyP),
			false,
		),
		Menu,
		MenuWithInputManager,
		MenuWithoutMouse,
	));
}
