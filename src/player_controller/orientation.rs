use bevy::prelude::*;

use crate::gravity::AffectedByGravity;

// FIXME: Move this to a different plugin

#[derive(Component)]
pub struct GravityOrientation;

pub fn orient(
	mut rigidbodies: Query<(&mut Transform, &AffectedByGravity), With<GravityOrientation>>,
) {
	for (mut transform, gravity) in rigidbodies.iter_mut() {
		transform.rotation =
			Quat::from_rotation_arc(transform.up().into(), gravity.up) * transform.rotation;
	}
}
