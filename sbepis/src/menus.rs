use std::time::Instant;

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_butler::*;
use leafwing_input_manager::plugin::{InputManagerPlugin, InputManagerSystem};
use leafwing_input_manager::prelude::{ActionState, InputMap};
use leafwing_input_manager::{Actionlike, InputControlKind};

use crate::input::InputManagerReference;

#[butler_plugin(build(
	register_type::<MenuStack>(),
	add_plugins(InputManagerMenuPlugin::<CloseMenuAction>::default()),
))]
pub struct MenusPlugin;

pub struct InputManagerMenuPlugin<Action: Actionlike>(std::marker::PhantomData<Action>);
impl<Action: Actionlike + TypePath + bevy::reflect::GetTypeRegistration> Plugin
	for InputManagerMenuPlugin<Action>
{
	fn build(&self, app: &mut App) {
		app.register_type::<ActionState<Action>>()
			.register_type::<InputMap<Action>>()
			.add_plugins(InputManagerPlugin::<Action>::default())
			.add_systems(
				PreUpdate,
				(
					enable_input_managers::<Action>,
					disable_input_managers::<Action>,
				)
					.in_set(InputManagerSystem::ManualControl),
			);
	}
}
impl<Action: Actionlike> Default for InputManagerMenuPlugin<Action> {
	fn default() -> Self {
		Self(default())
	}
}

#[derive(Component)]
pub struct Menu;

#[derive(Component)]
pub struct MenuWithInputManager;

#[derive(Component)]
pub struct MenuWithMouse;

#[derive(Component)]
pub struct MenuWithoutMouse;

#[derive(Component)]
pub struct MenuHidesWhenClosed;

#[derive(Component)]
pub struct MenuDespawnsWhenClosed;

#[derive(Resource, Default, Debug, Reflect)]
#[resource(plugin = MenusPlugin)]
pub struct MenuStack {
	stack: Vec<Entity>,
	current: Option<Entity>,
}
impl MenuStack {
	pub fn push(&mut self, menu: Entity) {
		self.stack.push(menu);
	}

	pub fn remove(&mut self, menu: Entity) {
		self.stack.retain(|&entity| entity != menu);
	}

	pub fn contains(&self, menu: Entity) -> bool {
		self.stack.contains(&menu)
	}

	pub fn toggle(&mut self, menu: Entity) {
		if self.contains(menu) {
			self.remove(menu);
		} else {
			self.push(menu);
		}
	}
}

#[derive(Event)]
#[event(plugin = MenusPlugin)]
pub struct MenuActivated(pub Entity);
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MenuActivatedSet;

#[derive(Event)]
#[event(plugin = MenusPlugin)]
pub struct MenuDeactivated(pub Entity);
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MenuDeactivatedSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MenuManipulationSet;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Reflect, Debug)]
pub struct CloseMenuAction;
impl Actionlike for CloseMenuAction {
	fn input_control_kind(&self) -> InputControlKind {
		InputControlKind::Button
	}
}
impl CloseMenuBinding for CloseMenuAction {
	type Action = Self;
	fn action() -> Self {
		Self
	}
}

#[system(
	plugin = MenusPlugin, schedule = Update,
	after = MenuManipulationSet,
	in_set = MenuActivatedSet,
	in_set = MenuDeactivatedSet,
	run_if = resource_changed::<MenuStack>,
)]
fn activate_stack_current(
	mut menu_stack: ResMut<MenuStack>,
	mut ev_activated: EventWriter<MenuActivated>,
	mut ev_deactivated: EventWriter<MenuDeactivated>,
) {
	if let Some(current) = menu_stack.current {
		if menu_stack.stack.last() != Some(&current) {
			ev_deactivated.send(MenuDeactivated(current));
			menu_stack.current = None;
		}
	}

	if menu_stack.current.is_none() && !menu_stack.stack.is_empty() {
		let new_current = *menu_stack.stack.last().unwrap();
		menu_stack.current = Some(new_current);
		ev_activated.send(MenuActivated(new_current));
	}
}

#[system(
	plugin = MenusPlugin, schedule = Update,
	after = MenuActivatedSet,
)]
fn show_mouse(
	mut ev_activated: EventReader<MenuActivated>,
	menus: Query<(), With<MenuWithMouse>>,
	mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
	let mut window = window.get_single_mut().expect("No primary window found");
	for MenuActivated(menu) in ev_activated.read() {
		if menus.get(*menu).is_ok() {
			window.cursor_options.grab_mode = CursorGrabMode::None;
			window.cursor_options.visible = true;
		}
	}
}

#[system(
	plugin = MenusPlugin, schedule = Update,
	after = MenuActivatedSet,
)]
fn hide_mouse(
	mut ev_activated: EventReader<MenuActivated>,
	menus: Query<(), With<MenuWithoutMouse>>,
	mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
	let mut window = window.get_single_mut().expect("No primary window found");
	for MenuActivated(menu) in ev_activated.read() {
		if menus.get(*menu).is_ok() {
			window.cursor_options.grab_mode = CursorGrabMode::Locked;
			window.cursor_options.visible = false;
		}
	}
}

fn enable_input_managers<Action: Actionlike>(
	mut ev_activated: EventReader<MenuActivated>,
	mut menus: Query<&mut ActionState<Action>, With<MenuWithInputManager>>,
) {
	for MenuActivated(menu) in ev_activated.read() {
		if let Ok(mut input_manager) = menus.get_mut(*menu) {
			input_manager.enable();

			// On the first frame of a new input manager, already held buttons
			// are "just pressed" so we need to clear them
			input_manager.tick(Instant::now(), Instant::now());
		}
	}
}

fn disable_input_managers<Action: Actionlike>(
	mut ev_deactivated: EventReader<MenuDeactivated>,
	mut menus: Query<&mut ActionState<Action>, With<MenuWithInputManager>>,
) {
	for MenuDeactivated(menu) in ev_deactivated.read() {
		if let Ok(mut input_manager) = menus.get_mut(*menu) {
			input_manager.disable();
		}
	}
}

pub trait CloseMenuBinding {
	type Action: Actionlike + Copy;
	fn action() -> Self::Action;
}
pub trait OpenMenuBinding {
	type Action: Actionlike + Copy;
	type Menu: Component;
	fn action() -> Self::Action;
}

#[system(
	plugin = MenusPlugin, schedule = Update,
	generics = CloseMenuAction,
	in_set = MenuManipulationSet,
)]
pub fn close_menu_on_action<Binding: CloseMenuBinding>(
	input: Query<(Entity, &ActionState<Binding::Action>)>,
	mut menu_stack: ResMut<MenuStack>,
) {
	for (entity, _) in input
		.iter()
		.filter(|(_, input)| input.just_pressed(&Binding::action()))
	{
		menu_stack.remove(entity);
	}
}

pub fn close_menu_on_event<Ev: Event + InputManagerReference>(
	mut menu_stack: ResMut<MenuStack>,
	mut ev_input: EventReader<Ev>,
) {
	for input_manager in ev_input.read() {
		menu_stack.remove(input_manager.input_manager());
	}
}

pub fn show_menu_on_action<Binding: OpenMenuBinding>(
	input: Query<&ActionState<Binding::Action>>,
	mut menus: Query<Entity, With<Binding::Menu>>,
	mut menu_stack: ResMut<MenuStack>,
) {
	for _ in input
		.iter()
		.filter(|input| input.just_pressed(&Binding::action()))
	{
		let menu = menus.get_single_mut().expect("Menu not found");
		menu_stack.push(menu);
	}
}

#[system(
	plugin = MenusPlugin, schedule = Update,
	after = MenuActivatedSet,
)]
fn show_menus(
	mut ev_activated: EventReader<MenuActivated>,
	mut menus: Query<&mut Visibility, With<MenuHidesWhenClosed>>,
) {
	for MenuActivated(menu) in ev_activated.read() {
		if let Ok(mut visibility) = menus.get_mut(*menu) {
			*visibility = Visibility::Visible;
		}
	}
}

#[system(
	plugin = MenusPlugin, schedule = Update,
	after = MenuDeactivatedSet,
)]
fn hide_menus(
	mut ev_deactivated: EventReader<MenuDeactivated>,
	mut menus: Query<&mut Visibility, With<MenuHidesWhenClosed>>,
) {
	for MenuDeactivated(menu) in ev_deactivated.read() {
		if let Ok(mut visibility) = menus.get_mut(*menu) {
			*visibility = Visibility::Hidden;
		}
	}
}

#[system(
	plugin = MenusPlugin, schedule = Update,
	after = MenuDeactivatedSet,
)]
fn despawn_menus(
	mut ev_deactivated: EventReader<MenuDeactivated>,
	mut menus: Query<Entity, With<MenuDespawnsWhenClosed>>,
	mut commands: Commands,
) {
	for MenuDeactivated(menu) in ev_deactivated.read() {
		if let Ok(menu) = menus.get_mut(*menu) {
			commands.entity(menu).despawn_recursive();
		}
	}
}
