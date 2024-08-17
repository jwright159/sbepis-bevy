mod movement;
mod orientation;
mod camera_controls;

use std::f32::consts::PI;
use self::movement::*;
use self::orientation::*;
use self::camera_controls::*;

pub use self::camera_controls::{PlayerCamera, PlayerBody, MouseSensitivity};

use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::gravity::GravityRigidbodyBundle;
use crate::gridbox_material;
use crate::input::button_just_pressed;
use crate::input::clamped_dual_axes_input;
use crate::input::dual_axes_input;
use crate::input::spawn_input_manager;

pub struct PlayerControllerPlugin;
impl Plugin for PlayerControllerPlugin
{
	fn build(&self, app: &mut App) {
		app
			.insert_resource(MouseSensitivity(0.003))
			.insert_resource(PlayerSpeed { speed: 5.0, sprint_modifier: 2.0, jump_speed: 5.0 })
			
			.add_plugins(InputManagerPlugin::<MovementAction>::default())
			
			.add_systems(Startup, (
				setup,
				spawn_input_manager(InputMap::default()
					.with_dual_axis(MovementAction::Move, KeyboardVirtualDPad::WASD)
					.with(MovementAction::Jump, KeyCode::Space)
					.with_dual_axis(MovementAction::Look, MouseMove::default())
					.with(MovementAction::Sprint, KeyCode::ShiftLeft)
				),
			))
			.add_systems(Update, (
				orient,
				dual_axes_input(MovementAction::Look).pipe(rotate_camera_and_body),
				clamped_dual_axes_input(MovementAction::Move).pipe(axes_to_ground_velocity).pipe(strafe),
				jump.run_if(button_just_pressed(MovementAction::Jump)),
			))
			;
	}
}

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
)
{
	let position = Vec3::new(5.0, 10.0, 0.0);
	
	let body = commands.spawn((
		Name::new("Player Body"),
		PbrBundle {
			transform: Transform::from_translation(position),
			mesh: meshes.add(Capsule3d::new(0.25, 1.0).mesh().rings(1).latitudes(8).longitudes(16).uv_profile(CapsuleUvProfile::Fixed)),
			material: gridbox_material("white", &mut materials, &asset_server),
			..default()
		},
		GravityRigidbodyBundle::default(),
		Collider::capsule_y(0.5, 0.25),
		GravityOrientation,
		PlayerBody,
		LockedAxes::ROTATION_LOCKED,
	)).id();
	
	commands.spawn((
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
	)).set_parent(body);
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Reflect, Debug)]
pub enum MovementAction
{
	Move,
	Jump,
	Look,
	Sprint,
}

impl Actionlike for MovementAction {
	fn input_control_kind(&self) -> InputControlKind {
		match self {
			MovementAction::Move => InputControlKind::DualAxis,
			MovementAction::Jump => InputControlKind::Button,
			MovementAction::Look => InputControlKind::DualAxis,
			MovementAction::Sprint => InputControlKind::Button,
		}
	}
}