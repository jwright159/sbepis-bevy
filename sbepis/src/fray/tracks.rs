use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use soundyrust::MidiAudioTrackHandle;

use crate::dialogue::spawn_dialogue;
use crate::menus::{InputManagerReference, MenuStack};

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct TrackSwitcher;

#[derive(Resource)]
pub struct FrayTracks {
	pub player: Track,
	pub imp: Track,
	pub four_four: MidiAudioTrackHandle,
	pub six_eight: MidiAudioTrackHandle,
}
impl FrayTracks {
	pub fn player_track(&self) -> MidiAudioTrackHandle {
		self.track(self.player)
	}

	pub fn imp_track(&self) -> MidiAudioTrackHandle {
		self.track(self.imp)
	}

	fn track(&self, track: Track) -> MidiAudioTrackHandle {
		match track {
			Track::FourFour => self.four_four,
			Track::SixEight => self.six_eight,
		}
	}

	pub fn set_player_track(&mut self, track: Track) {
		self.player = track;
		self.imp = match track {
			Track::FourFour => Track::SixEight,
			Track::SixEight => Track::FourFour,
		};
	}
}

pub fn open_track_switch_dialogue(
	_switcher_entity: In<Entity>,
	mut commands: Commands,
	mut menu_stack: ResMut<MenuStack>,
) {
	let mut dialogue = spawn_dialogue(
		&mut commands,
		&mut menu_stack,
		"Select a track for the player to use.\nThe imps will use the other one.".to_owned(),
		(),
		InputMap::<TrackSwitcherAction>::default(),
	);
	dialogue.add_option(
		&mut commands,
		"4/4".to_owned(),
		TrackSwitcherFourFour {
			dialogue: dialogue.root,
		},
	);
	dialogue.add_option(
		&mut commands,
		"6/8".to_owned(),
		TrackSwitcherSixEight {
			dialogue: dialogue.root,
		},
	);
}

pub fn switch_track(
	mut ev_track_switched: EventReader<TrackSwitched>,
	mut fray_tracks: ResMut<FrayTracks>,
) {
	for ev in ev_track_switched.read() {
		fray_tracks.set_player_track(ev.track);
	}
}

#[derive(Component)]
pub struct TrackSwitcherFourFour {
	pub dialogue: Entity,
}
impl InputManagerReference for TrackSwitcherFourFour {
	fn input_manager(&self) -> Entity {
		self.dialogue
	}
}

#[derive(Component)]
pub struct TrackSwitcherSixEight {
	pub dialogue: Entity,
}
impl InputManagerReference for TrackSwitcherSixEight {
	fn input_manager(&self) -> Entity {
		self.dialogue
	}
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Reflect, Debug)]
pub enum TrackSwitcherAction {
	FourFour,
	SixEight,
}
impl Actionlike for TrackSwitcherAction {
	fn input_control_kind(&self) -> InputControlKind {
		match self {
			TrackSwitcherAction::FourFour => InputControlKind::Button,
			TrackSwitcherAction::SixEight => InputControlKind::Button,
		}
	}
}

#[derive(Clone, Copy)]
pub enum Track {
	FourFour,
	SixEight,
}

#[derive(Event, Clone, Copy)]
pub struct TrackSwitched {
	pub track: Track,
	pub dialogue: Entity,
}
impl InputManagerReference for TrackSwitched {
	fn input_manager(&self) -> Entity {
		self.dialogue
	}
}
