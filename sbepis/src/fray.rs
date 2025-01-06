use std::f32::consts::TAU;
use std::time::Duration;

use bevy::audio::Volume;
use bevy::prelude::*;
use soundyrust::*;

use crate::camera::PlayerCameraNode;
use crate::util::MapRange;

pub struct FrayPlugin;

impl Plugin for FrayPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins(SoundyPlugin)
			.add_systems(Startup, play_background_music)
			.add_systems(Update, tick_fray_music);
	}
}

fn play_background_music(mut commands: Commands, mut assets: ResMut<Assets<MidiAudio>>) {
	commands.spawn((
		AudioPlayer(
			assets.add(
				MidiAudio::from_bytes(
					include_bytes!("../assets/fray.mid"),
					include_bytes!("../assets/hl4mgm.sf2"),
				)
				.with_channel_patch(0, 0, 46)
				.with_channel_patch(1, 0, 3)
				.with_channel_patch(2, 128, 0)
				.with_channel_patch(3, 0, 0),
			),
		),
		PlaybackSettings::LOOP
			.with_volume(Volume::new(0.2))
			.paused(),
		Name::new("Background Music"),
		FrayMusic::default(),
	));

	commands.spawn((
		Name::new("Beat Counter"),
		BeatCounter::default(),
		Text("".to_owned()),
		Node {
			position_type: PositionType::Absolute,
			bottom: Val::Px(5.0),
			left: Val::Px(5.0),
			..default()
		},
		PlayerCameraNode,
	));
}

#[derive(Component)]
pub struct FrayMusic {
	beat: f64,
	beats_per_second: f64,
	delay: Option<Duration>,
}

impl FrayMusic {
	fn default() -> Self {
		Self {
			beat: 0.0,
			beats_per_second: 0.0,
			delay: Some(Duration::from_secs_f32(1.0)),
		}
	}

	pub fn tick(&mut self, delta: Duration, midi_audio: &MidiAudio) {
		self.beats_per_second = midi_audio.beats_per_second() / 2.0;
		self.beat += self.time_to_bpm_beat(delta);
	}

	pub fn subbeats(&self, divisions: u32) -> u32 {
		(self.beat * divisions as f64).floor() as u32
	}

	pub fn beat_progress(&self) -> f32 {
		self.beat.fract() as f32
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

	pub fn time_to_bpm_beat(&self, time: Duration) -> f64 {
		time.as_secs_f64() * self.beats_per_second
	}

	pub fn speed(&self) -> f32 {
		self.beats_per_second as f32
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
	mut fray_musics: Query<(&mut FrayMusic, &AudioSink, &AudioPlayer<MidiAudio>)>,
	mut beat_counters: Query<(&mut BeatCounter, &mut Text)>,
	assets: Res<Assets<MidiAudio>>,
) {
	let (mut beat_counter, mut beat_counter_text) = beat_counters
		.get_single_mut()
		.expect("Couldn't find beat counter");
	for (mut fray_music, audio_sink, midi_audio) in fray_musics.iter_mut() {
		if let Some(delay) = fray_music.delay {
			fray_music.delay = delay.checked_sub(time.delta());

			#[cfg(feature = "metronome")]
			if fray_music.delay.is_none() {
				commands.spawn((
					Name::new("Beat"),
					AudioPlayer::new(asset_server.load("metronome.mp3")),
					PlaybackSettings::DESPAWN,
				));
			}

			continue;
		}

		let midi_audio = assets.get(&midi_audio.0).expect("Couldn't find midi audio");
		audio_sink.play(); // this should really be phased out or smth
		fray_music.tick(time.delta(), midi_audio);
		let beat = fray_music.subbeats(1);
		let beat_progress = fray_music.beat_progress();

		beat_counter_text.0 = format!("{} {:.2}", beat, beat_progress);

		#[cfg(feature = "metronome")]
		if beat_counter.beat != beat {
			commands.spawn((
				Name::new("Beat"),
				AudioPlayer::new(asset_server.load("metronome.mp3")),
				PlaybackSettings::DESPAWN.with_speed(if beat % 4 == 0 { 1.0 } else { 0.5 }),
			));
		}

		beat_counter.beat = beat;
	}
}
