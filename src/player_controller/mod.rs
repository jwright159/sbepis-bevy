use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::gridbox_material;
use crate::input::*;
use crate::main_bundles::EntityBundle;

use self::camera_controls::*;
pub use self::camera_controls::{MouseSensitivity, PlayerBody, PlayerCamera};
use self::movement::*;
use self::movement::{axes_to_ground_velocity, jump};
use self::weapons::hammer::*;
use self::weapons::rifle::*;
use self::weapons::sword::*;
use self::weapons::*;

mod camera_controls;
mod movement;
mod weapons;

pub struct PlayerControllerPlugin;
impl Plugin for PlayerControllerPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(MouseSensitivity(0.003))
			.insert_resource(PlayerSpeed {
				speed: 5.0,
				sprint_modifier: 2.0,
				jump_speed: 5.0,
			})
			.add_event::<DamageEvent>()
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
							.with(PlayerAction::Use, MouseButton::Left)
							.with(PlayerAction::NextWeapon, MouseScrollDirection::UP)
							.with(PlayerAction::PrevWeapon, MouseScrollDirection::DOWN),
					),
				),
			)
			.add_systems(
				Update,
				(
					dual_axes_input(PlayerAction::Look).pipe(rotate_camera_and_body),
					clamped_dual_axes_input(PlayerAction::Move).pipe(axes_to_ground_velocity),
					jump::<PlayerBody>.run_if(button_just_pressed(PlayerAction::Jump)),
					attack.run_if(button_just_pressed(PlayerAction::Use)),
					switch_weapon_next.run_if(button_just_pressed(PlayerAction::NextWeapon)),
					switch_weapon_prev.run_if(button_just_pressed(PlayerAction::PrevWeapon)),
					initialize_weapon_sets,
					animate_hammer,
					animate_sword,
					animate_rifle,
					sweep_dealers,
					deal_all_damage,
					update_damage_numbers,
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
			EntityBundle::new(
				Transform::from_translation(Vec3::new(5.0, 10.0, 0.0)),
				meshes.add(
					Capsule3d::new(0.25, 1.0)
						.mesh()
						.rings(1)
						.latitudes(8)
						.longitudes(16)
						.uv_profile(CapsuleUvProfile::Fixed),
				),
				gridbox_material("white", &mut materials, &asset_server),
				Collider::capsule_y(0.5, 0.25),
			),
			PlayerBody,
		))
		.id();

	let camera = commands
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
		.set_parent(body)
		.id();

	let (hammer_pivot, _hammer_head) = spawn_hammer(
		&mut commands,
		&asset_server,
		&mut materials,
		&mut meshes,
		body,
	);

	let (sword_pivot, _sword_blade) = spawn_sword(
		&mut commands,
		&asset_server,
		&mut materials,
		&mut meshes,
		body,
	);

	let (rifle_pivot, _rifle_barrel) = spawn_rifle(
		&mut commands,
		&asset_server,
		&mut materials,
		&mut meshes,
		body,
	);

	commands.entity(body).insert((
		WeaponSet {
			weapons: vec![sword_pivot, hammer_pivot, rifle_pivot],
			active_weapon: 0,
		},
		UninitializedWeaponSet,
	));

	commands.spawn((
		Name::new("Damage Numbers"),
		TextBundle::from_section("Damage", TextStyle::default()).with_style(Style {
			position_type: PositionType::Absolute,
			bottom: Val::Px(5.0),
			right: Val::Px(5.0),
			..default()
		}),
		DamageNumbers,
		TargetCamera(camera),
	));

	commands.spawn((
		Name::new("Debug Collider Visualizer"),
		DebugColliderVisualizer,
		SpatialBundle::default(),
		CollisionGroups::new(Group::NONE, Group::NONE),
	));
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Reflect, Debug)]
pub enum PlayerAction {
	Move,
	Jump,
	Look,
	Sprint,
	Use,
	NextWeapon,
	PrevWeapon,
}

impl Actionlike for PlayerAction {
	fn input_control_kind(&self) -> InputControlKind {
		match self {
			PlayerAction::Move => InputControlKind::DualAxis,
			PlayerAction::Jump => InputControlKind::Button,
			PlayerAction::Look => InputControlKind::DualAxis,
			PlayerAction::Sprint => InputControlKind::Button,
			PlayerAction::Use => InputControlKind::Button,
			PlayerAction::NextWeapon => InputControlKind::Button,
			PlayerAction::PrevWeapon => InputControlKind::Button,
		}
	}
}
