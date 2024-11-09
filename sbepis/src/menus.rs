use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use leafwing_input_manager::plugin::InputManagerPlugin;
use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::{Actionlike, InputControlKind};

pub struct MenusPlugin;
impl Plugin for MenusPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<MenuStack>()
			.init_resource::<MenuStack>()
			.add_event::<MenuActivated>()
			.add_event::<MenuDeactivated>()
			.add_systems(
				Update,
				(
					activate_stack_current.run_if(resource_changed::<MenuStack>),
					show_mouse,
					hide_mouse,
				),
			);
	}
}

pub struct InputManagerMenuPlugin<Action: Actionlike>(std::marker::PhantomData<Action>);
impl<Action: Actionlike + TypePath + bevy::reflect::GetTypeRegistration> Plugin
	for InputManagerMenuPlugin<Action>
{
	fn build(&self, app: &mut App) {
		app.add_plugins(InputManagerPlugin::<Action>::default())
			.add_systems(
				Update,
				(
					enable_input_managers::<Action>,
					disable_input_managers::<Action>,
				),
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

#[derive(Resource, Default, Debug, Reflect)]
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
pub struct MenuActivated(pub Entity);

#[derive(Event)]
pub struct MenuDeactivated(pub Entity);

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

fn show_mouse(
	mut ev_activated: EventReader<MenuActivated>,
	menus: Query<(), With<MenuWithMouse>>,
	mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
	let mut window = window.get_single_mut().expect("No primary window found");
	for MenuActivated(menu) in ev_activated.read() {
		if menus.get(*menu).is_ok() {
			window.cursor.grab_mode = CursorGrabMode::None;
			window.cursor.visible = true;
		}
	}
}

fn hide_mouse(
	mut ev_activated: EventReader<MenuActivated>,
	menus: Query<(), With<MenuWithoutMouse>>,
	mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
	let mut window = window.get_single_mut().expect("No primary window found");
	for MenuActivated(menu) in ev_activated.read() {
		if menus.get(*menu).is_ok() {
			window.cursor.grab_mode = CursorGrabMode::Locked;
			window.cursor.visible = false;
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

#[derive(Clone, Copy, Eq, PartialEq, Hash, Reflect, Debug)]
pub enum MenuAction {
	CloseMenu,
}
impl Actionlike for MenuAction {
	fn input_control_kind(&self) -> InputControlKind {
		match self {
			MenuAction::CloseMenu => InputControlKind::Button,
		}
	}
}
