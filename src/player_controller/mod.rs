use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::gravity::GravityRigidbodyBundle;
use crate::gridbox_material;
use crate::input::*;

use self::camera_controls::*;
pub use self::camera_controls::{MouseSensitivity, PlayerBody, PlayerCamera};
pub use self::interaction::Health;
use self::interaction::*;
use self::movement::*;
pub use self::movement::{axes_to_ground_velocity, jump, strafe};
pub use self::orientation::GravityOrientation;
use self::orientation::*;

mod camera_controls;
mod interaction;
mod movement;
mod orientation;

pub struct PlayerControllerPlugin;
impl Plugin for PlayerControllerPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(MouseSensitivity(0.003))
			.insert_resource(PlayerSpeed {
				speed: 5.0,
				sprint_modifier: 2.0,
				jump_speed: 5.0,
			})
			.add_plugins(InputManagerPlugin::<PlayerAction>::default())
			.add_systems(
				Startup,
				(
					setup,
					spawn_input_manager(
						InputMap::default()
							.with_dual_axis(PlayerAction::Move, KeyboardVirtualDPad::WASD)
							.with(PlayerAction::Jump, KeyCode::Space)
							.with_dual_axis(PlayerAction::Look, MouseMove::default())
							.with(PlayerAction::Sprint, KeyCode::ShiftLeft)
							.with(PlayerAction::Use, MouseButton::Left),
					),
				),
			)
			.add_systems(
				Update,
				(
					orient,
					dual_axes_input(PlayerAction::Look).pipe(rotate_camera_and_body),
					clamped_dual_axes_input(PlayerAction::Move)
						.pipe(axes_to_ground_velocity)
						.pipe(wrap_velocity_in_hashmap)
						.pipe(strafe::<PlayerBody>),
					jump::<PlayerBody>.run_if(button_just_pressed(PlayerAction::Jump)),
					attack.run_if(button_just_pressed(PlayerAction::Use)),
					animate_hammer,
					collide_hammer,
					kill_entities_with_no_health,
				),
			);
	}
}

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
) {
	let body = commands
		.spawn((
			Name::new("Player Body"),
			PbrBundle {
				transform: Transform::from_translation(Vec3::new(5.0, 10.0, 0.0)),
				mesh: meshes.add(
					Capsule3d::new(0.25, 1.0)
						.mesh()
						.rings(1)
						.latitudes(8)
						.longitudes(16)
						.uv_profile(CapsuleUvProfile::Fixed),
				),
				material: gridbox_material("white", &mut materials, &asset_server),
				..default()
			},
			GravityRigidbodyBundle::default(),
			Collider::capsule_y(0.5, 0.25),
			GravityOrientation,
			PlayerBody,
			LockedAxes::ROTATION_LOCKED,
		))
		.id();

	commands
		.spawn((
			Name::new("Player Camera"),
			Camera3dBundle {
				transform: Transform::from_translation(Vec3::Y * 0.5),
				projection: Projection::Perspective(PerspectiveProjection {
					fov: 70.0 / 180. * PI,
					..default()
				}),
				..default()
			},
			PlayerCamera,
			Pitch(0.0),
		))
		.set_parent(body);

	let hammer_pivot = commands
		.spawn((
			Name::new("Hammer Pivot"),
			TransformBundle::from_transform(Transform::from_translation(Vec3::ZERO)),
			VisibilityBundle::default(),
			HammerPivot,
		))
		.set_parent(body)
		.id();

	commands
		.spawn((
			Name::new("Hammer Head"),
			PbrBundle {
				transform: Transform::default()
					.with_translation(Vec3::Y * 1.)
					.with_rotation(Quat::from_rotation_x(PI / 2.)),
				mesh: meshes.add(
					Capsule3d::new(0.1, 0.5)
						.mesh()
						.rings(1)
						.latitudes(8)
						.longitudes(16)
						.uv_profile(CapsuleUvProfile::Fixed),
				),
				material: gridbox_material("red", &mut materials, &asset_server),
				..default()
			},
			Collider::capsule_y(0.25, 0.1),
			Sensor,
			ActiveEvents::COLLISION_EVENTS,
			Hammer {
				damage: 1.0,
				pivot: hammer_pivot,
			},
		))
		.set_parent(hammer_pivot);
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Reflect, Debug)]
pub enum PlayerAction {
	Move,
	Jump,
	Look,
	Sprint,
	Use,
}

impl Actionlike for PlayerAction {
	fn input_control_kind(&self) -> InputControlKind {
		match self {
			PlayerAction::Move => InputControlKind::DualAxis,
			PlayerAction::Jump => InputControlKind::Button,
			PlayerAction::Look => InputControlKind::DualAxis,
			PlayerAction::Sprint => InputControlKind::Button,
			PlayerAction::Use => InputControlKind::Button,
		}
	}
}
