use core::panic;
use std::collections::VecDeque;
use std::f32::consts::TAU;
use std::time::Duration;

use augmented_midi::{
	parse_midi_file, MIDIFile, MIDIFileChunk, MIDIFileDivision, MIDITrackEvent, MIDITrackInner,
};
use bevy::prelude::*;
use helgoboss_midi::{ShortMessage, ShortMessageFactory, StructuredShortMessage};
use notation_midi::prelude::*;
use notation_model::prelude::*;

use crate::camera::PlayerCameraNode;
use crate::util::MapRange;

pub struct FrayPlugin;

impl Plugin for FrayPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<SwitchTabEvent>()
			.add_event::<JumpToBarEvent>()
			.add_event::<PlayControlEvent>()
			.add_plugins((MidiPlugin,))
			.add_systems(
				Startup,
				play_background_music.after(StereoStream::setup_default_streaming),
			)
			.add_systems(FixedUpdate, tick_fray_music);
	}
}

fn play_background_music(
	mut commands: Commands,
	source: Query<Entity, With<Handle<StereoStream>>>,
) {
	let midi_bytes = include_bytes!("../assets/fray.mid");
	let (_, midi_file) =
		parse_midi_file::<String, Vec<u8>>(midi_bytes).expect("Failed to parse MIDI file");
	let midi_track = MidiTrack::from_midi_file(midi_file);

	commands
		.entity(source.get_single().expect("Couldn't find audio source"))
		.insert((Name::new("Background Music"), FrayMusic::new(midi_track)));

	commands.spawn((
		Name::new("Beat Counter"),
		BeatCounter::default(),
		TextBundle::from_section("", TextStyle::default()).with_style(Style {
			position_type: PositionType::Absolute,
			bottom: Val::Px(5.0),
			left: Val::Px(5.0),
			..default()
		}),
		PlayerCameraNode,
	));
}

#[derive(Component)]
pub struct FrayMusic {
	midi_track: MidiTrack,
	tick: f64,
	event_index: usize,
	beat: f32,
	bpm: f64,
	delay: Option<Duration>,
}

impl FrayMusic {
	fn new(midi_track: MidiTrack) -> Self {
		Self {
			midi_track,
			tick: 0.0,
			event_index: 0,
			beat: 0.0,
			bpm: 120.0,
			delay: Some(Duration::from_secs_f32(1.0)),
		}
	}

	pub fn tick(&mut self, time: &Time, hub: &mut MidiHub) {
		let delta_ticks =
			self.midi_track.ticks_per_beat as f64 * self.bpm / 60.0 * time.delta_seconds_f64();
		self.tick += delta_ticks;

		while let Some(event) = self
			.midi_track
			.events
			.get(self.event_index)
			.filter(|event| event.time <= self.tick as u64)
		{
			match event.inner {
				MidiEvent::Meta(MidiMetaEvent::Tempo { tempo }) => {
					self.bpm = tempo;
				}
				MidiEvent::Message(message) => {
					hub.send(
						&MidiSettings::default(),
						&PlaySpeed::new(&TabMeta::default()),
						&MidiMessage::new(
							EntryPassMode::Immediate,
							BarPosition::ZERO,
							Units::MIN_ACCURACY,
							false,
							message,
						),
						message.data_byte_2().into(),
					);
				}
			}
			self.event_index += 1;
		}

		let delta_beat = self.time_to_bpm_beat(time.delta());
		self.beat += delta_beat;

		if self.event_index >= self.midi_track.events.len() {
			self.event_index = 0;
			self.tick -= self.midi_track.events.last().unwrap().time as f64;
		}
	}

	pub fn subbeats(&self, divisions: u32) -> u32 {
		(self.beat * divisions as f32).floor() as u32
	}

	pub fn beat_progress(&self) -> f32 {
		self.beat.fract()
	}

	pub fn modify_fray_damage(&self, damage: f32) -> f32 {
		let modifier = self.single_beat_modifier(1.0)
			+ self.single_beat_modifier(2.0)
			+ self.single_beat_modifier(4.0)
			+ self.single_beat_modifier(8.0);
		damage * modifier
	}

	fn single_beat_modifier(&self, factor: f32) -> f32 {
		(self.beat_progress() * factor)
			.map_range(0.0..1.0, 0.0..TAU)
			.cos()
			.map_range(-1.0..1.0, 0.0..1.0)
			/ factor
	}

	pub fn time_to_bpm_beat(&self, time: Duration) -> f32 {
		(time.as_secs_f64() * self.fray_bpm() / 60.0) as f32
	}

	pub fn fray_bpm(&self) -> f64 {
		self.bpm * 0.5
	}
}

#[derive(Component, Default)]
pub struct BeatCounter {
	pub beat: u32,
}

fn tick_fray_music(
	#[cfg(feature = "metronome")] mut commands: Commands,
	#[cfg(feature = "metronome")] asset_server: Res<AssetServer>,
	time: Res<Time>,
	mut fray_musics: Query<(&mut FrayMusic, &AudioSink)>,
	mut beat_counters: Query<(&mut BeatCounter, &mut Text)>,
	mut hub: NonSendMut<MidiHub>,
) {
	let (mut beat_counter, mut beat_counter_text) = beat_counters
		.get_single_mut()
		.expect("Couldn't find beat counter");
	for (mut fray_music, audio_sink) in fray_musics.iter_mut() {
		if let Some(delay) = fray_music.delay {
			fray_music.delay = delay.checked_sub(time.delta());

			#[cfg(feature = "metronome")]
			if fray_music.delay.is_none() {
				commands.spawn((
					Name::new("Beat"),
					AudioBundle {
						source: asset_server.load("metronome.mp3"),
						settings: PlaybackSettings::DESPAWN,
					},
				));
			}

			continue;
		}

		audio_sink.play();
		fray_music.tick(&time, &mut hub);
		let beat = fray_music.subbeats(1);
		let beat_progress = fray_music.beat_progress();

		beat_counter_text.sections[0].value = format!("{} {:.2}", beat, beat_progress);

		#[cfg(feature = "metronome")]
		if beat_counter.beat != beat {
			commands.spawn((
				Name::new("Beat"),
				AudioBundle {
					source: asset_server.load("metronome.mp3"),
					settings: PlaybackSettings::DESPAWN.with_speed(if beat % 4 == 0 {
						1.0
					} else {
						0.5
					}),
				},
			));
		}

		beat_counter.beat = beat;
	}
}

#[derive(Debug, Clone)]
pub struct MidiTrackAccumulateEvent {
	pub time: u64,
	pub inner: MidiEvent,
}

#[derive(Debug, Clone)]
pub struct MidiTrack {
	pub events: Vec<MidiTrackAccumulateEvent>,
	pub ticks_per_beat: u16,
}

impl MidiTrack {
	pub fn from_midi_file(file: MIDIFile<String, Vec<u8>>) -> Self {
		let mut events = Vec::new();
		let mut time = 0_u64;
		let mut tracks: Vec<VecDeque<MIDITrackEvent<Vec<u8>>>> = file
			.chunks
			.iter()
			.filter_map(|chunk| match chunk {
				MIDIFileChunk::Track { events } => Some(events.iter().cloned().collect()),
				_ => None,
			})
			.collect();

		while tracks.iter().any(|track| !track.is_empty()) {
			let next_event_track_index = tracks
				.iter()
				.enumerate()
				.filter_map(|(i, track)| track.front().map(|event| (i, event)))
				.min_by_key(|(_, event)| event.delta_time())
				.map(|(i, _)| i)
				.unwrap();
			let next_event = tracks[next_event_track_index].pop_front().unwrap();
			let inner = match next_event.inner {
				MIDITrackInner::Message(message) => {
					let bytes = Vec::<u8>::from(message);
					MidiEvent::Message(
						StructuredShortMessage::from_bytes((
							bytes[0],
							bytes
								.get(1)
								.copied()
								.unwrap_or_default()
								.try_into()
								.expect("Data 1 high bit set"),
							bytes
								.get(2)
								.copied()
								.unwrap_or_default()
								.try_into()
								.expect("Data 2 high bit set"),
						))
						.expect("Failed to parse MIDI message"),
					)
				}
				MIDITrackInner::Meta(meta) => match meta.meta_type {
					0x51 => {
						let microseconds_per_beat =
							u32::from_be_bytes([0, meta.bytes[0], meta.bytes[1], meta.bytes[2]]);
						let tempo = 60_000_000.0 / microseconds_per_beat as f64;
						MidiEvent::Meta(MidiMetaEvent::Tempo { tempo })
					}
					_ => continue,
				},
			};
			time += next_event.delta_time as u64;
			events.push(MidiTrackAccumulateEvent { time, inner });

			for track in tracks
				.iter_mut()
				.enumerate()
				.filter(|(i, _)| *i != next_event_track_index)
				.map(|(_, track)| track)
			{
				let mut remaining_time = next_event.delta_time;
				for event in track.iter_mut() {
					if remaining_time == 0 {
						break;
					}

					let sub = event.delta_time.min(remaining_time);
					event.delta_time -= sub;
					remaining_time -= sub;
				}
			}
		}

		Self {
			events,
			ticks_per_beat: match file
				.header()
				.expect("MIDI file must have a header chunk")
				.division
			{
				MIDIFileDivision::TicksPerQuarterNote {
					ticks_per_quarter_note,
				} => ticks_per_quarter_note,
				_ => panic!("Invalid MIDI file division"),
			},
		}
	}
}

#[derive(Debug, Clone)]
pub enum MidiEvent {
	Meta(MidiMetaEvent),
	Message(StructuredShortMessage),
}

#[derive(Debug, Clone)]
pub enum MidiMetaEvent {
	Tempo { tempo: f64 },
}
