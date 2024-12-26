use std::f32::consts::PI;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;
use leafwing_input_manager::prelude::*;

use crate::camera::PlayerCamera;
use crate::gridbox_material;
use crate::input::*;
use crate::inventory::Inventory;
use crate::main_bundles::EntityBundle;
use crate::menus::{
	InputManagerMenuPlugin, Menu, MenuStack, MenuWithInputManager, MenuWithoutMouse,
};

use self::camera_controls::*;
pub use self::camera_controls::{interact_with, MouseSensitivity, PlayerBody};
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
				speed: 7.0,
				sprint_modifier: 2.0,
				jump_speed: 5.0,
				friction: 6.0,
				acceleration: 8.0,
				air_acceleration: 6.0,
			})
			.add_event::<EntityDamaged>()
			.add_plugins(InputManagerMenuPlugin::<PlayerAction>::default())
			.add_systems(Startup, setup)
			.add_systems(
				Update,
				(
					dual_axes_input(PlayerAction::Look).pipe(rotate_camera_and_body),
					clamped_dual_axes_input(PlayerAction::Move).pipe(axes_to_ground_velocity),
					jump.run_if(button_just_pressed(PlayerAction::Jump)),
					attack.run_if(button_just_pressed(PlayerAction::Use)),
					switch_weapon_next.run_if(button_just_pressed(PlayerAction::NextWeapon)),
					switch_weapon_prev.run_if(button_just_pressed(PlayerAction::PrevWeapon)),
					initialize_weapon_sets,
					animate_hammer,
					animate_sword,
					animate_rifle,
					charge_rifle,
					sweep_dealers,
					deal_all_damage,
					update_damage_numbers,
					update_is_grounded,
				),
			);
	}
}

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
	mut menu_stack: ResMut<MenuStack>,
) {
	let input = commands
		.spawn((
			input_manager_bundle(
				InputMap::default()
					.with_dual_axis(PlayerAction::Move, VirtualDPad::wasd())
					.with(PlayerAction::Jump, KeyCode::Space)
					.with_dual_axis(PlayerAction::Look, MouseMove::default())
					.with(PlayerAction::Sprint, KeyCode::ShiftLeft)
					.with(PlayerAction::Use, MouseButton::Left)
					.with(PlayerAction::Interact, KeyCode::KeyE)
					.with(PlayerAction::NextWeapon, MouseScrollDirection::UP)
					.with(PlayerAction::PrevWeapon, MouseScrollDirection::DOWN)
					.with(PlayerAction::OpenQuestScreen, KeyCode::KeyJ)
					.with(PlayerAction::OpenInventory, KeyCode::KeyV),
				false,
			),
			Menu,
			MenuWithInputManager,
			MenuWithoutMouse,
		))
		.id();
	menu_stack.push(input);

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
				Collider::capsule(0.25, 1.0),
			),
			PlayerBody { is_grounded: false },
			Inventory::default(),
		))
		.id();

	let camera = commands
		.spawn((
			Name::new("Player Camera"),
			Camera3d::default(),
			Transform::from_translation(Vec3::Y * 0.5),
			Projection::Perspective(PerspectiveProjection {
				fov: 70.0 / 180. * PI,
				..default()
			}),
			PlayerCamera,
			Pitch(0.0),
			RayCaster::new(Vec3::ZERO, Dir3::Z)
				.with_max_hits(1)
				.with_solidness(false),
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
		Text("Damage".to_owned()),
		Node {
			position_type: PositionType::Absolute,
			bottom: Val::Px(5.0),
			right: Val::Px(5.0),
			..default()
		},
		DamageNumbers,
		TargetCamera(camera),
	));

	commands.spawn((
		Name::new("Debug Collider Visualizer"),
		DebugColliderVisualizer,
	));
}

fn update_is_grounded(
	mut bodies: Query<(Entity, &mut PlayerBody, &GlobalTransform)>,
	spatial_query: SpatialQuery,
) {
	for (entity, mut body, transform) in bodies.iter_mut() {
		body.is_grounded = spatial_query
			.shape_intersections(
				&Collider::sphere(0.25),
				transform.translation() - transform.rotation() * Vec3::Y * 0.5,
				Quat::IDENTITY,
				&SpatialQueryFilter::default(),
			)
			.into_iter()
			.any(|collided_entity| collided_entity != entity);
	}
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Reflect, Debug)]
pub enum PlayerAction {
	Move,
	Jump,
	Look,
	Sprint,
	Use,
	Interact,
	NextWeapon,
	PrevWeapon,
	OpenQuestScreen,
	OpenInventory,
}
impl Actionlike for PlayerAction {
	fn input_control_kind(&self) -> InputControlKind {
		match self {
			PlayerAction::Move => InputControlKind::DualAxis,
			PlayerAction::Jump => InputControlKind::Button,
			PlayerAction::Look => InputControlKind::DualAxis,
			PlayerAction::Sprint => InputControlKind::Button,
			PlayerAction::Use => InputControlKind::Button,
			PlayerAction::Interact => InputControlKind::Button,
			PlayerAction::NextWeapon => InputControlKind::Button,
			PlayerAction::PrevWeapon => InputControlKind::Button,
			PlayerAction::OpenQuestScreen => InputControlKind::Button,
			PlayerAction::OpenInventory => InputControlKind::Button,
		}
	}
}
