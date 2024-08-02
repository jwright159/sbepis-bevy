mod notes;
mod commands;
mod staff;
mod note_holder;

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::input::action_event;
use crate::input::button_event;
use crate::input::spawn_input_manager_with_bindings;

use self::note_holder::*;
use self::notes::*;
use self::commands::*;
use self::staff::*;

pub struct PlayerCommandsPlugin;

impl Plugin for PlayerCommandsPlugin
{
	fn build(&self, app: &mut App) {
		app
			.add_plugins(InputManagerPlugin::<ToggleStaffAction>::default())
			.add_plugins(InputManagerPlugin::<PlayNoteAction>::default())
			.insert_resource(ToggleActions::<PlayNoteAction>::DISABLED)
			
			.add_event::<NotePlayedEvent>()
			.add_event::<CommandSentEvent>()
			.add_event::<ClearNotesEvent>()
			.add_event::<ToggleStaffEvent>()

			.add_event::<PingCommandEvent>()
			.add_event::<KillCommandEvent>()

			.insert_resource(NotePatternPlayer::default())

			.add_systems(Startup, (
				spawn_staff,
				
				spawn_input_manager_with_bindings([
					(PlayNoteAction::C4, KeyCode::KeyZ),
					(PlayNoteAction::CS4, KeyCode::KeyS),
					(PlayNoteAction::D4, KeyCode::KeyX),
					(PlayNoteAction::DS4, KeyCode::KeyD),
					(PlayNoteAction::E4, KeyCode::KeyC),
					(PlayNoteAction::F4, KeyCode::KeyV),
					(PlayNoteAction::FS4, KeyCode::KeyG),
					(PlayNoteAction::G4, KeyCode::KeyB),
					(PlayNoteAction::GS4, KeyCode::KeyH),
					(PlayNoteAction::A4, KeyCode::KeyN),
					(PlayNoteAction::AS4, KeyCode::KeyJ),
					(PlayNoteAction::B4, KeyCode::KeyM),
					
					(PlayNoteAction::C5, KeyCode::Comma),
					(PlayNoteAction::CS5, KeyCode::KeyL),
					(PlayNoteAction::D5, KeyCode::Period),
					(PlayNoteAction::DS5, KeyCode::Semicolon),
					(PlayNoteAction::E5, KeyCode::Slash),
					
					(PlayNoteAction::C5, KeyCode::KeyQ),
					(PlayNoteAction::CS5, KeyCode::Digit2),
					(PlayNoteAction::D5, KeyCode::KeyW),
					(PlayNoteAction::DS5, KeyCode::Digit3),
					(PlayNoteAction::E5, KeyCode::KeyE),
					(PlayNoteAction::F5, KeyCode::KeyR),
					(PlayNoteAction::FS5, KeyCode::Digit5),
					(PlayNoteAction::G5, KeyCode::KeyT),
					(PlayNoteAction::GS5, KeyCode::Digit6),
					(PlayNoteAction::A5, KeyCode::KeyY),
					(PlayNoteAction::AS5, KeyCode::Digit7),
					(PlayNoteAction::B5, KeyCode::KeyU),
					
					(PlayNoteAction::C6, KeyCode::KeyI),
					(PlayNoteAction::CS6, KeyCode::Digit9),
					(PlayNoteAction::D6, KeyCode::KeyO),
					(PlayNoteAction::DS6, KeyCode::Digit0),
					(PlayNoteAction::E6, KeyCode::KeyP),
				]),
				spawn_input_manager_with_bindings([
					(ToggleStaffAction::ToggleStaff, KeyCode::Backquote),
				])
			))
			
			.add_systems(PostStartup, (
				#[cfg(feature = "spawn_debug_notes_on_staff")]
				spawn_debug_notes,
				setup_staff_camera,
			))

			.add_systems(PreUpdate, (
				action_event(|action: PlayNoteAction| NotePlayedEvent(action.note())),
				button_event(ToggleStaffAction::ToggleStaff, ToggleStaffEvent::default)
			))

			.add_systems(Update, (
				(
					toggle_staff,
					(
						show_staff,
						enable_note_input,
						disable_movement_input,
					).run_if(is_staff_open),
					(
						hide_staff,
						disable_note_input,
						enable_movement_input,
						send_clear_notes,
					).run_if(not(is_staff_open)),
				).chain().run_if(on_event::<ToggleStaffEvent>()),

				(
					spawn_note_audio,
					add_note_to_holder,
					add_note_to_player,
				).run_if(on_event::<NotePlayedEvent>()),
				(
					check_note_patterns::<PingCommandEvent>,
					check_note_patterns::<KillCommandEvent>,
				),
				(
					clear_notes.run_if(on_event::<CommandSentEvent>()),
					ping.run_if(on_event::<PingCommandEvent>()),
					kill.run_if(on_event::<KillCommandEvent>()),
				),
				(
					clear_holder_notes,
					clear_player_notes,
				).run_if(on_event::<ClearNotesEvent>()),
			).chain())

			;
	}
}