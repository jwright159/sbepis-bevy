use std::f32::consts::PI;

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::entity::movement::AimRotators;
use crate::input::*;
use crate::netcode::ClientPlayer;

use self::movement::*;
pub use self::movement::{MouseAim, PlayerBody, PlayerHead};
use self::weapons::hammer::*;
use self::weapons::rifle::*;
use self::weapons::sword::*;
pub use self::weapons::WeaponSet;
use self::weapons::*;

mod movement;
mod weapons;

pub struct PlayerControllerPlugin;
impl Plugin for PlayerControllerPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<DamageEvent>().add_systems(
			Update,
			(
				setup_heads,
				setup_weapon_sets,
				update_player_aim,
				animate_hammer,
				animate_sword,
				animate_rifle,
				charge_rifle,
				sweep_dealers,
				deal_all_damage,
				update_damage_numbers,
			),
		);
	}
}

pub struct SpawnPlayerPlugin;
impl Plugin for SpawnPlayerPlugin {
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
			)
			.add_systems(
				Update,
				(
					setup_client,
					dual_axes_input(PlayerAction::Look).pipe(mouse_input),
					clamped_dual_axes_input(PlayerAction::Move).pipe(movement_input),
					jump::<PlayerBody>.run_if(button_just_pressed(PlayerAction::Jump)),
					attack.run_if(button_just_pressed(PlayerAction::Use)),
					switch_weapon_next.run_if(button_just_pressed(PlayerAction::NextWeapon)),
					switch_weapon_prev.run_if(button_just_pressed(PlayerAction::PrevWeapon)),
				),
			);
	}
}

fn setup_heads(mut commands: Commands, bodies: Query<Entity, Added<PlayerBody>>) {
	for body in bodies.iter() {
		let head = commands
			.spawn((
				Name::new("Player Head"),
				SpatialBundle::from_transform(Transform::from_translation(Vec3::Y * 0.5)),
				PlayerHead,
			))
			.set_parent(body)
			.id();

		commands.entity(body).insert(AimRotators {
			body,
			head: Some(head),
		});
	}
}

fn setup_client(
	mut commands: Commands,
	bodies: Query<&AimRotators, (Added<AimRotators>, With<ClientPlayer>)>,
) {
	for AimRotators { body, head } in bodies.iter() {
		let [body, head] = [*body, head.unwrap()];

		commands.entity(body).insert(MouseAim::default());

		commands.entity(head).insert((
			ClientPlayer,
			Camera3dBundle {
				projection: Projection::Perspective(PerspectiveProjection {
					fov: 70.0 / 180. * PI,
					..default()
				}),
				transform: Transform::from_translation(Vec3::Y * 0.5),
				..default()
			},
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
			TargetCamera(head),
		));
	}
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
