use bevy::ecs::schedule::SystemConfigs;
use bevy::prelude::*;
use bevy_rapier3d::math::Real;
use interpolation::{Ease, EaseFunction};
use std::array::IntoIter;
use std::ops::Range;

use crate::camera::PlayerCamera;
use crate::iter_system::IntoIteratorSystemTrait;
use crate::player_controller::PlayerBody;

pub trait MapRange<T> {
	fn map_range(self, range_in: Range<T>, range_out: Range<T>) -> T;
	fn map_to_01(self, range_in: Range<T>) -> T;
	fn map_from_01(self, range_out: Range<T>) -> T;
	fn map_range_ease(self, range_in: Range<T>, range_out: Range<T>, ease: EaseFunction) -> T;
}
impl MapRange<Real> for Real {
	fn map_range(self, range_in: Range<Real>, range_out: Range<Real>) -> Real {
		self.map_to_01(range_in).map_from_01(range_out)
	}

	fn map_to_01(self, range_in: Range<Real>) -> Real {
		(self - range_in.start) / (range_in.end - range_in.start)
	}

	fn map_from_01(self, range_out: Range<Real>) -> Real {
		self * (range_out.end - range_out.start) + range_out.start
	}

	fn map_range_ease(
		self,
		range_in: Range<Real>,
		range_out: Range<Real>,
		ease: EaseFunction,
	) -> Real {
		self.map_to_01(range_in).calc(ease).map_from_01(range_out)
	}
}

pub trait TransformEx {
	fn transform_vector3(&self, vector: Vec3) -> Vec3;
	fn inverse_transform_point(&self, point: Vec3) -> Vec3;
	#[allow(dead_code)]
	fn inverse_transform_vector3(&self, vector: Vec3) -> Vec3;
}
impl TransformEx for GlobalTransform {
	fn transform_vector3(&self, vector: Vec3) -> Vec3 {
		self.affine().transform_vector3(vector)
	}

	fn inverse_transform_point(&self, point: Vec3) -> Vec3 {
		self.affine().inverse().transform_point3(point)
	}

	fn inverse_transform_vector3(&self, vector: Vec3) -> Vec3 {
		self.affine().inverse().transform_vector3(vector)
	}
}

pub trait IterElements<T, const N: usize> {
	fn iter_elements(&self) -> IntoIter<T, N>;
}
impl IterElements<f32, 3> for Vec3 {
	fn iter_elements(&self) -> IntoIter<f32, 3> {
		[self.x, self.y, self.z].into_iter()
	}
}

#[derive(Component, Deref, DerefMut)]
pub struct DespawnTimer(Timer);

impl DespawnTimer {
	pub fn new(duration: f32) -> Self {
		Self(Timer::from_seconds(duration, TimerMode::Once))
	}
}

pub fn despawn_after_timer(
	mut commands: Commands,
	time: Res<Time>,
	mut query: Query<(Entity, &mut DespawnTimer)>,
) {
	for (entity, mut despawn_timer) in query.iter_mut() {
		despawn_timer.tick(time.delta());
		if despawn_timer.finished() {
			commands.entity(entity).despawn_recursive();
		}
	}
}

#[derive(Component)]
#[require(Transform)]
pub struct Billboard;

pub fn billboard(
	mut transforms: Query<&mut Transform, With<Billboard>>,
	player_camera: Query<&GlobalTransform, With<PlayerCamera>>,
	player_body: Query<&GlobalTransform, With<PlayerBody>>,
) {
	let player_camera = player_camera.get_single().expect("No player camera found");
	let player_body = player_body.get_single().expect("No player body found");
	for mut transform in transforms.iter_mut() {
		transform.look_at(player_camera.translation(), player_body.up());
	}
}

pub trait QuaternionEx {
	fn from_look_at(position: Vec3, target: Vec3, up: impl TryInto<Dir3>) -> Quat;
	fn from_look_to(direction: impl TryInto<Dir3>, up: impl TryInto<Dir3>) -> Quat;
}

impl QuaternionEx for Quat {
	fn from_look_at(position: Vec3, target: Vec3, up: impl TryInto<Dir3>) -> Quat {
		Self::from_look_to(target - position, up)
	}

	fn from_look_to(direction: impl TryInto<Dir3>, up: impl TryInto<Dir3>) -> Quat {
		let back = -direction.try_into().unwrap_or(Dir3::NEG_Z);
		let up = up.try_into().unwrap_or(Dir3::Y);
		let right = up
			.cross(back.into())
			.try_normalize()
			.unwrap_or_else(|| up.any_orthonormal_vector());
		let up = back.cross(right);
		Quat::from_mat3(&Mat3::from_cols(right, up, back.into()))
	}
}

pub fn map_event<EventA: Event + Clone, EventB: Event, Marker>(
	event_generator: impl IntoSystem<In<EventA>, EventB, Marker>, // TODO: This should probably be InRef
) -> SystemConfigs {
	(move |mut ev_a: EventReader<EventA>| ev_a.read().cloned().collect::<Vec<EventA>>())
		.iter_map(event_generator)
		.pipe(
			|In(events): In<Vec<EventB>>, mut ev_b: EventWriter<EventB>| {
				for event in events.into_iter() {
					ev_b.send(event);
				}
			},
		)
		.into_configs()
}

#[macro_export]
macro_rules! some_or_return {
	($value:expr) => {
		match $value {
			Some(value) => value,
			None => return,
		}
	};
}

#[macro_export]
macro_rules! some_or_continue {
	($value:expr) => {
		match $value {
			Some(value) => value,
			None => continue,
		}
	};
}

#[macro_export]
macro_rules! ok_or_return {
	($value:expr) => {
		match $value {
			Ok(value) => value,
			Err(_) => return,
		}
	};
}

#[macro_export]
macro_rules! ok_or_continue {
	($value:expr) => {
		match $value {
			Ok(value) => value,
			Err(_) => continue,
		}
	};
}
