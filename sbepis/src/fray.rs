use std::f32::consts::TAU;
use std::io::Cursor;
use std::sync::Arc;
use std::time::Duration;

use async_channel::{Receiver, Sender, TryRecvError};
use augmented_midi::{
	parse_midi_file, MIDIFile, MIDIFileChunk, MIDIMessage, MIDIMetaEvent, MIDITrackEvent,
	MIDITrackInner,
};
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::audio::{AddAudioSource, Source};
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use itertools::Itertools;
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};

use crate::camera::PlayerCameraNode;
use crate::util::MapRange;

pub struct FrayPlugin;

impl Plugin for FrayPlugin {
	fn build(&self, app: &mut App) {
		app.add_audio_source::<MidiAudio>()
			.init_asset::<MidiAudio>()
			.init_asset_loader::<MidiAssetLoader>()
			.add_systems(Startup, play_background_music)
			.add_systems(Update, tick_fray_music);
	}
}

fn play_background_music(mut commands: Commands, asset_server: Res<AssetServer>) {
	let midi_bytes = include_bytes!("../assets/fray.mid");
	let (_, midi_file) =
		parse_midi_file::<String, Vec<u8>>(midi_bytes).expect("Failed to parse MIDI file");
	for event in midi_file.chunks.iter() {
		match event {
			MIDIFileChunk::Header(header) => {
				println!("{:?}", header);
			}
			MIDIFileChunk::Track { events } => {
				for event in events {
					println!("{:?}", event);
				}
			}
			MIDIFileChunk::Unknown { name, body } => {
				println!("{:?}: {:?}", name, body);
			}
		}
	}

	let midi_file = MidiFile::new(midi_file);
	let midi_audio = MidiAudio::new(midi_file);

	commands.spawn((
		Name::new("Background Music"),
		FrayMusic::new(&midi_audio),
		AudioSourceBundle {
			source: asset_server.add::<MidiAudio>(midi_audio),
			settings: PlaybackSettings::LOOP,
		},
	));
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
	beat: f32,
	bpm: f32,
	tx: Sender<FrayToMidiEvent>,
	rx: Receiver<MidiToFrayEvent>,
}

impl FrayMusic {
	fn new(midi_audio: &MidiAudio) -> Self {
		Self {
			beat: 0.0,
			bpm: 0.0,
			tx: midi_audio.fray_to_midi_tx.clone(),
			rx: midi_audio.midi_to_fray_rx.clone(),
		}
	}

	pub fn tick(&mut self, time: &Time) {
		if self.bpm == 0.0 {
			self.tx
				.try_send(FrayToMidiEvent::SetSpeed(1.0))
				.expect("Failed to set speed");
		}

		while let Ok(event) = self.rx.try_recv() {
			match event {
				MidiToFrayEvent::SetTempo(bpm) => {
					self.bpm = bpm * 0.5;
				}
			}
		}

		let delta_beat = self.time_to_bpm_beat(time.delta());
		self.beat += delta_beat;
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
		time.as_secs_f32() * self.bpm / 60.0
	}
}

#[derive(Component, Default)]
pub struct BeatCounter {
	pub beat: u32,
}

fn tick_fray_music(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	time: Res<Time>,
	mut fray_musics: Query<&mut FrayMusic>,
	mut beat_counters: Query<(&mut BeatCounter, &mut Text)>,
) {
	let (mut beat_counter, mut beat_counter_text) = beat_counters
		.get_single_mut()
		.expect("Couldn't find beat counter");
	for mut fray_music in fray_musics.iter_mut() {
		fray_music.tick(&time);
		let beat = fray_music.subbeats(1);
		let beat_progress = fray_music.beat_progress();

		beat_counter_text.sections[0].value = format!("{} {:.2}", beat, beat_progress);

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

pub enum FrayToMidiEvent {
	SetSpeed(f32),
}

pub enum MidiToFrayEvent {
	SetTempo(f32),
}

// Most of the following is stolen from https://github.com/exvacuum/bevy_rustysynth/blob/master/src/assets.rs

/// MIDI audio asset
#[derive(Asset, TypePath, Debug, Clone)]
struct MidiAudio {
	midi_file: MidiFile,
	fray_to_midi_tx: Sender<FrayToMidiEvent>,
	fray_to_midi_rx: Receiver<FrayToMidiEvent>,
	midi_to_fray_tx: Sender<MidiToFrayEvent>,
	midi_to_fray_rx: Receiver<MidiToFrayEvent>,
}

impl MidiAudio {
	pub fn new(midi_file: MidiFile) -> Self {
		let (fray_to_midi_tx, fray_to_midi_rx) = async_channel::unbounded::<FrayToMidiEvent>();
		let (midi_to_fray_tx, midi_to_fray_rx) = async_channel::unbounded::<MidiToFrayEvent>();
		Self {
			midi_file,
			fray_to_midi_tx,
			fray_to_midi_rx,
			midi_to_fray_tx,
			midi_to_fray_rx,
		}
	}
}

/// AssetLoader for MIDI files (.mid/.midi)
#[derive(Default, Debug)]
struct MidiAssetLoader;

impl AssetLoader for MidiAssetLoader {
	type Asset = MidiAudio;

	type Settings = ();

	type Error = std::io::Error;

	async fn load<'a>(
		&'a self,
		reader: &'a mut Reader<'_>,
		_settings: &'a Self::Settings,
		_load_context: &'a mut LoadContext<'_>,
	) -> Result<Self::Asset, Self::Error> {
		let mut bytes = vec![];
		reader.read_to_end(&mut bytes).await?;
		let (_, midi_file) =
			parse_midi_file::<String, Vec<u8>>(&bytes).expect("Failed to parse MIDI file");
		let midi_file = MidiFile::new(midi_file);
		Ok(MidiAudio::new(midi_file))
	}

	fn extensions(&self) -> &[&str] {
		&["mid", "midi"]
	}
}

/// Decoder for MIDI file playback
struct MidiFileDecoder {
	sample_rate: usize,
	stream: Receiver<f32>,
}

impl MidiFileDecoder {
	/// Construct and begin a new MIDI sequencer with the given MIDI data and soundfont.
	///
	/// The sequencer will push at most 1 second's worth of audio ahead, allowing the decoder to
	/// be paused without endlessly backing up data forever.
	fn new(
		midi: MidiAudio,
		soundfont: Arc<SoundFont>,
		event_tx: Sender<MidiToFrayEvent>,
		event_rx: Receiver<FrayToMidiEvent>,
	) -> Self {
		let sample_rate = 44100_usize;
		let (tx, rx) = async_channel::bounded::<f32>(sample_rate * 2);
		AsyncComputeTaskPool::get()
			.spawn(async move {
				let settings = SynthesizerSettings::new(sample_rate as i32);
				let synthesizer =
					Synthesizer::new(&soundfont, &settings).expect("Failed to create synthesizer");
				let mut sequencer = MidiFileSequencer::new(synthesizer, event_tx, event_rx);
				let midi = Arc::new(midi.midi_file);
				sequencer.play(&midi, true);
				let mut left: Vec<f32> = vec![0_f32; sample_rate];
				let mut right: Vec<f32> = vec![0_f32; sample_rate];
				while !sequencer.end_of_sequence() {
					sequencer.render(&mut left, &mut right);
					for value in left.iter().interleave(right.iter()) {
						if (tx.send(*value).await).is_err() {
							return;
						};
					}
				}

				tx.close();
			})
			.detach();
		Self {
			sample_rate,
			stream: rx,
		}
	}
}

impl Iterator for MidiFileDecoder {
	type Item = f32;

	fn next(&mut self) -> Option<Self::Item> {
		match self.stream.try_recv() {
			Ok(value) => Some(value),
			Err(e) => match e {
				TryRecvError::Empty => Some(0.0),
				TryRecvError::Closed => None,
			},
		}
	}
}

impl Source for MidiFileDecoder {
	fn current_frame_len(&self) -> Option<usize> {
		None
	}

	fn channels(&self) -> u16 {
		2
	}

	fn sample_rate(&self) -> u32 {
		self.sample_rate as u32
	}

	fn total_duration(&self) -> Option<std::time::Duration> {
		None
	}
}

static SOUNDFONT: &[u8] = include_bytes!("../assets/hl4mgm.sf2");

impl Decodable for MidiAudio {
	type Decoder = MidiFileDecoder;

	type DecoderItem = <MidiFileDecoder as Iterator>::Item;

	fn decoder(&self) -> Self::Decoder {
		MidiFileDecoder::new(
			self.clone(),
			Arc::new(
				SoundFont::new(&mut Cursor::new(SOUNDFONT)).expect("Failed to load soundfont"),
			),
			self.midi_to_fray_tx.clone(),
			self.fray_to_midi_rx.clone(),
		)
	}
}

// Most of the following is stolen from https://github.com/sinshu/rustysynth/blob/main/rustysynth/src/midifile.rs but using augmented_midi because pub(crate) SUCKS

#[derive(Debug, Clone)]
pub struct MidiFile {
	messages: Vec<MIDITrackEvent<Vec<u8>>>,
	times: Vec<f64>,
}

impl MidiFile {
	pub fn new(midi_file: MIDIFile<String, Vec<u8>>) -> Self {
		let mut message_lists: Vec<Vec<MIDITrackEvent<Vec<u8>>>> = Vec::new();
		let mut tick_lists: Vec<Vec<i32>> = Vec::new();

		for chunk in midi_file.chunks.iter() {
			let (message_list, tick_list) = MidiFile::read_track(chunk);
			message_lists.push(message_list);
			tick_lists.push(tick_list);
		}

		let (messages, times) = MidiFile::merge_tracks(
			&message_lists,
			&tick_lists,
			midi_file.ticks_per_quarter_note() as i32,
		);
		Self { messages, times }
	}

	fn read_track(
		chunk: &MIDIFileChunk<String, Vec<u8>>,
	) -> (Vec<MIDITrackEvent<Vec<u8>>>, Vec<i32>) {
		let MIDIFileChunk::Track { events } = chunk else {
			return (Vec::new(), Vec::new());
		};

		let mut messages: Vec<MIDITrackEvent<Vec<u8>>> = Vec::new();
		let mut ticks: Vec<i32> = Vec::new();

		let mut tick: i32 = 0;

		for event in events {
			let delta = event.delta_time as i32;

			tick += delta;

			match &event.inner {
				MIDITrackInner::Meta(meta) => {
					match meta.meta_type {
						0x2F => {
							// End of Track
							messages.push(event.clone());
							ticks.push(tick);

							// Some MIDI files may have events inserted after the EOT.
							// Such events should be ignored.
							return (messages, ticks);
						}
						0x51 => {
							// Set Tempo
							messages.push(event.clone());
							ticks.push(tick);
						}
						_ => (),
					};
				}
				MIDITrackInner::Message(message) => {
					match message {
						MIDIMessage::SysExMessage(_) => (),
						MIDIMessage::Other { .. } => (),
						_ => {
							messages.push(event.clone());
							ticks.push(tick);
						}
					};
				}
			};
		}

		panic!("End of track not found");
	}

	fn merge_tracks(
		message_lists: &[Vec<MIDITrackEvent<Vec<u8>>>],
		tick_lists: &[Vec<i32>],
		resolution: i32,
	) -> (Vec<MIDITrackEvent<Vec<u8>>>, Vec<f64>) {
		let mut merged_messages: Vec<MIDITrackEvent<Vec<u8>>> = Vec::new();
		let mut merged_times: Vec<f64> = Vec::new();

		let mut indices: Vec<usize> = vec![0; message_lists.len()];

		let mut current_tick: i32 = 0;
		let mut current_time: f64 = 0.0;

		let mut tempo: f64 = 120.0;

		loop {
			let mut min_tick = i32::MAX;
			let mut min_index: i32 = -1;

			for ch in 0..tick_lists.len() {
				if indices[ch] < tick_lists[ch].len() {
					let tick = tick_lists[ch][indices[ch]];
					if tick < min_tick {
						min_tick = tick;
						min_index = ch as i32;
					}
				}
			}

			if min_index == -1 {
				break;
			}

			let next_tick = tick_lists[min_index as usize][indices[min_index as usize]];
			let delta_tick = next_tick - current_tick;
			let delta_time = 60.0 / (resolution as f64 * tempo) * delta_tick as f64;

			current_tick += delta_tick;
			current_time += delta_time;

			let message = message_lists[min_index as usize][indices[min_index as usize]].clone();
			match &message.inner {
				MIDITrackInner::Meta(inner) if inner.meta_type == 0x51 => {
					tempo = meta_to_tempo(inner);
					merged_messages.push(message);
					merged_times.push(current_time);
				}
				_ => {
					merged_messages.push(message);
					merged_times.push(current_time);
				}
			}

			indices[min_index as usize] += 1;
		}

		(merged_messages, merged_times)
	}
}

fn meta_to_tempo(inner: &MIDIMetaEvent<Vec<u8>>) -> f64 {
	60.0 / ((inner.bytes[0] as f64 * 65536.0
		+ inner.bytes[1] as f64 * 256.0
		+ inner.bytes[2] as f64)
		/ 1000000.0)
}

// Most of the following is stolen from https://github.com/sinshu/rustysynth/blob/main/rustysynth/src/midifile_sequencer.rs

/// An instance of the MIDI file sequencer.
#[non_exhaustive]
pub struct MidiFileSequencer {
	synthesizer: Synthesizer,

	speed: f64,

	midi_file: Option<Arc<MidiFile>>,
	play_loop: bool,

	block_wrote: usize,

	current_time: f64,
	msg_index: usize,
	loop_index: usize,

	tx: Sender<MidiToFrayEvent>,
	rx: Receiver<FrayToMidiEvent>,
}

impl MidiFileSequencer {
	/// Initializes a new instance of the sequencer.
	///
	/// # Arguments
	///
	/// * `synthesizer` - The synthesizer to be handled by the sequencer.
	pub fn new(
		synthesizer: Synthesizer,
		tx: Sender<MidiToFrayEvent>,
		rx: Receiver<FrayToMidiEvent>,
	) -> Self {
		Self {
			synthesizer,
			speed: 1.0,
			midi_file: None,
			play_loop: false,
			block_wrote: 0,
			current_time: 0.0,
			msg_index: 0,
			loop_index: 0,
			tx,
			rx,
		}
	}

	/// Plays the MIDI file.
	///
	/// # Arguments
	///
	/// * `midi_file` - The MIDI file to be played.
	/// * `play_loop` - If `true`, the MIDI file loops after reaching the end.
	pub fn play(&mut self, midi_file: &Arc<MidiFile>, play_loop: bool) {
		self.midi_file = Some(Arc::clone(midi_file));
		self.play_loop = play_loop;

		self.block_wrote = self.synthesizer.get_block_size();

		self.current_time = 0.0;
		self.msg_index = 0;
		self.loop_index = 0;

		self.synthesizer.reset()
	}

	/// Stops playing.
	#[allow(dead_code)]
	pub fn stop(&mut self) {
		self.midi_file = None;
		self.synthesizer.reset();
	}

	/// Renders the waveform.
	///
	/// # Arguments
	///
	/// * `left` - The buffer of the left channel to store the rendered waveform.
	/// * `right` - The buffer of the right channel to store the rendered waveform.
	///
	/// # Remarks
	///
	/// The output buffers for the left and right must be the same length.
	pub fn render(&mut self, left: &mut [f32], right: &mut [f32]) {
		if left.len() != right.len() {
			panic!("The output buffers for the left and right must be the same length.");
		}

		while let Ok(event) = self.rx.try_recv() {
			match event {
				FrayToMidiEvent::SetSpeed(speed) => {
					self.speed = speed as f64;
				}
			}
		}

		let left_length = left.len();
		let mut wrote: usize = 0;
		while wrote < left_length {
			if self.block_wrote == self.synthesizer.get_block_size() {
				self.process_events();
				self.block_wrote = 0;
				self.current_time += self.speed * self.synthesizer.get_block_size() as f64
					/ self.synthesizer.get_sample_rate() as f64;
			}

			let src_rem = self.synthesizer.get_block_size() - self.block_wrote;
			let dst_rem = left_length - wrote;
			let rem = std::cmp::min(src_rem, dst_rem);

			self.synthesizer.render(
				&mut left[wrote..wrote + rem],
				&mut right[wrote..wrote + rem],
			);

			self.block_wrote += rem;
			wrote += rem;
		}
	}

	fn process_events(&mut self) {
		let midi_file = match self.midi_file.as_ref() {
			Some(value) => value,
			None => return,
		};

		while self.msg_index < midi_file.messages.len() {
			let time = midi_file.times[self.msg_index];
			let msg = &midi_file.messages[self.msg_index];

			if time <= self.current_time {
				match &msg.inner {
					MIDITrackInner::Message(message) => {
						let msg = Vec::<u8>::from(message.clone());
						let command = msg[0] & 0xF0;
						let channel = msg[0] & 0x0F;
						let data1 = msg.get(1).copied().unwrap_or(0);
						let data2 = msg.get(2).copied().unwrap_or(0);

						self.synthesizer.process_midi_message(
							channel as i32,
							command as i32,
							data1 as i32,
							data2 as i32,
						);
					}
					MIDITrackInner::Meta(meta) if meta.meta_type == 0x51 => {
						// Set Tempo
						self.tx
							.try_send(MidiToFrayEvent::SetTempo(meta_to_tempo(meta) as f32))
							.expect("Failed to set tempo");
					}
					MIDITrackInner::Meta(meta) if meta.meta_type == 0x52 => {
						// Loop Start
						self.loop_index = self.msg_index;
					}
					MIDITrackInner::Meta(meta) if meta.meta_type == 0x53 => {
						// Loop End
						self.current_time = midi_file.times[self.loop_index];
						self.msg_index = self.loop_index;
						self.synthesizer.note_off_all(false);
					}
					MIDITrackInner::Meta(_) => (),
				}

				self.msg_index += 1;
			} else {
				break;
			}
		}

		if self.msg_index == midi_file.messages.len() && self.play_loop {
			self.current_time = midi_file.times[self.loop_index];
			self.msg_index = self.loop_index;
			self.synthesizer.note_off_all(false);
		}
	}

	/// Gets the synthesizer handled by the sequencer.
	#[allow(dead_code)]
	pub fn get_synthesizer(&self) -> &Synthesizer {
		&self.synthesizer
	}

	/// Gets the currently playing MIDI file.
	#[allow(dead_code)]
	pub fn get_midi_file(&self) -> Option<&MidiFile> {
		match &self.midi_file {
			None => None,
			Some(value) => Some(value),
		}
	}

	/// Gets the current playback position in seconds.
	#[allow(dead_code)]
	pub fn get_position(&self) -> f64 {
		self.current_time
	}

	/// Gets a value that indicates whether the current playback position is at the end of the sequence.
	///
	/// # Remarks
	///
	/// If the `play` method has not yet been called, this value will be `true`.
	/// This value will never be `true` if loop playback is enabled.
	pub fn end_of_sequence(&self) -> bool {
		match &self.midi_file {
			None => true,
			Some(value) => self.msg_index == value.messages.len(),
		}
	}

	/// Gets the current playback speed.
	///
	/// # Remarks
	///
	/// The default value is 1.
	/// The tempo will be multiplied by this value during playback.
	#[allow(dead_code)]
	pub fn get_speed(&self) -> f64 {
		self.speed
	}

	/// Sets the playback speed.
	///
	/// # Remarks
	///
	/// The value must be non-negative.
	#[allow(dead_code)]
	pub fn set_speed(&mut self, value: f64) {
		if value < 0.0 {
			panic!("The playback speed must be a non-negative value.");
		}

		self.speed = value;
	}
}
