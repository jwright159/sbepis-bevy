use std::f32::consts::PI;
use bevy::prelude::*;

#[derive(Component)]
pub struct HammerPivot;

pub fn attack(
	In(attacking): In<bool>,
	mut hammer_pivot: Query<&mut Transform, With<HammerPivot>>,
) {
	let mut hammer_pivot = hammer_pivot.single_mut();
	hammer_pivot.rotation = if attacking { Quat::from_rotation_x(-PI / 2.) } else { Quat::IDENTITY };
}