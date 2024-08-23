use std::f32::consts::TAU;
use std::time::Duration;

use bevy::prelude::*;

use crate::player_controller::PlayerCamera;
use crate::util::MapRange;

pub struct FrayPlugin;

impl Plugin for FrayPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<FrayMusic>()
			.add_systems(Startup, play_background_music)
			.add_systems(PostStartup, setup_beat_counter_camera)
			.add_systems(Update, tick_fray_music);
	}
}

fn play_background_music(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.spawn((
		Name::new("Background Music"),
		AudioBundle {
			source: asset_server.load("Tutorial - Friday Night Funkin' OST.mp3"),
			settings: PlaybackSettings::LOOP,
		},
		FrayMusic {
			bpm: 100.0,
			offset: Duration::from_secs_f32(0.3),
			time: Duration::ZERO,
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
	));
}

pub fn setup_beat_counter_camera(
	mut commands: Commands,
	beat_counters: Query<Entity, With<BeatCounter>>,
	camera: Query<Entity, With<PlayerCamera>>,
) {
	let camera = camera.single();
	for beat_counter in beat_counters.iter() {
		commands.entity(beat_counter).insert(TargetCamera(camera));
	}
}

#[derive(Component, Reflect)]
pub struct FrayMusic {
	pub bpm: f32,
	pub offset: Duration,
	pub time: Duration,
}

impl FrayMusic {
	pub fn tick(&mut self, time: &Time) {
		self.time += time.delta();
	}

	pub fn beat_duration(&self) -> f32 {
		60.0 / self.bpm
	}

	fn adjusted_time(&self) -> f32 {
		(self.time.as_secs_f32() - self.offset.as_secs_f32()).max(0.0)
	}

	pub fn beat(&self) -> u32 {
		(self.adjusted_time() / self.beat_duration()).floor() as u32
	}

	pub fn beat_progress(&self) -> f32 {
		let beat = self.beat() as f32;
		let beat_time = beat as f32 * self.beat_duration();
		let beat_progress = (self.adjusted_time() - beat_time) / self.beat_duration();
		beat_progress
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
		time.as_secs_f32() / self.beat_duration()
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
		let beat = fray_music.beat();
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
