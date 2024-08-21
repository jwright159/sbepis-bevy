use bevy::prelude::*;
use num_traits::Float;
use std::ops::Range;
use std::{
	array::IntoIter,
	ops::{Add, Div, Mul, Sub},
};

use crate::player_controller::{PlayerBody, PlayerCamera};

pub trait MapRange<T> {
	fn map_range(self, range_in: Range<T>, range_out: Range<T>) -> T;
}
impl<T, F> MapRange<T> for F
where
	T: Add<Output = T> + Sub<Output = T> + Div<Output = T> + Mul<Output = T> + Copy,
	F: Float + Sub<T, Output = T>,
{
	fn map_range(self, range_in: Range<T>, range_out: Range<T>) -> T {
		(self - range_in.start) / (range_in.end - range_in.start)
			* (range_out.end - range_out.start)
			+ range_out.start
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
