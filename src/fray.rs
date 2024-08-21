use std::time::Duration;

use bevy::prelude::*;

use crate::player_controller::PlayerCamera;

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
		fray_music.time += time.delta();
		let beat_duration = 60.0 / fray_music.bpm;
		let beat_total_time =
			(fray_music.time.as_secs_f32() - fray_music.offset.as_secs_f32()).max(0.0);
		let beat = (beat_total_time / beat_duration).floor() as u32;
		let beat_time = beat as f32 * beat_duration;
		let beat_progress = (beat_total_time - beat_time) / beat_duration;

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
