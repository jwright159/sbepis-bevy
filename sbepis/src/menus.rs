use std::time::Instant;

use bevy::ecs::schedule::SystemConfigs;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use leafwing_input_manager::plugin::{InputManagerPlugin, InputManagerSystem};
use leafwing_input_manager::prelude::{ActionState, InputMap};
use leafwing_input_manager::{Actionlike, InputControlKind};

use crate::input::input_managers_where_button_just_pressed;
use crate::iter_system::{IntoDoneSystemTrait, IntoIteratorSystemTrait};

pub struct MenusPlugin;
impl Plugin for MenusPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<MenuStack>()
			.register_type::<ActionState<MenuAction>>()
			.register_type::<InputMap<MenuAction>>()
			.init_resource::<MenuStack>()
			.add_event::<MenuActivated>()
			.add_event::<MenuDeactivated>()
			.add_plugins(InputManagerMenuPlugin::<MenuAction>::default())
			.add_systems(
				Update,
				(
					activate_stack_current.run_if(resource_changed::<MenuStack>),
					show_mouse,
					hide_mouse,
					hide_menus,
					despawn_menus,
					close_menu_on(MenuAction::CloseMenu),
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
			window.cursor_options.grab_mode = CursorGrabMode::None;
			window.cursor_options.visible = true;
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

pub fn close_menu(In(menu): In<Entity>, mut menu_stack: ResMut<MenuStack>) {
	menu_stack.remove(menu);
}

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

pub fn show_menu<T: Component>(
	mut menus: Query<(Entity, &mut Visibility), With<T>>,
	mut menu_stack: ResMut<MenuStack>,
) {
	let (quest_screen, mut visibility) = menus
		.get_single_mut()
		.expect("Single menu with marker not found");
	*visibility = Visibility::Inherited;
	menu_stack.push(quest_screen);
}

pub fn close_menu_on<Action: Actionlike + Copy>(action: Action) -> SystemConfigs {
	input_managers_where_button_just_pressed(action)
		.iter_map(close_menu)
		.iter_done()
		.into_configs()
}

pub fn close_menu_on_event<Ev: Event + InputManagerReference>() -> SystemConfigs {
	|mut ev_input: EventReader<Ev>| -> Vec<Entity> {
		ev_input.read().map(|event| event.input_manager()).collect()
	}
	.iter_map(close_menu)
	.iter_done()
	.into_configs()
}

pub fn fire_input_events<Action: Actionlike + Copy, Ev: Event + InputManagerReference>(
	action: Action,
	event_generator: impl Fn(Entity) -> Ev + Send + Sync + 'static,
) -> SystemConfigs {
	input_managers_where_button_just_pressed(action)
		.iter_map(move |In(input_manager), mut ev_action: EventWriter<Ev>| {
			ev_action.send(event_generator(input_manager));
		})
		.iter_done()
		.into_configs()
}

pub fn fire_button_events<
	Button: Component + InputManagerReference,
	Ev: Event + InputManagerReference,
>(
	event_generator: impl Fn(Entity) -> Ev + Send + Sync + 'static,
) -> SystemConfigs {
	(move |buttons: Query<(&Button, &Interaction), Changed<Interaction>>,
	       mut ev_action: EventWriter<Ev>| {
		buttons
			.iter()
			.filter(|(_, &interaction)| interaction == Interaction::Pressed)
			.for_each(|(button, _)| {
				ev_action.send(event_generator(button.input_manager()));
			});
	})
	.into_configs()
}

pub fn fire_input_and_button_events<
	Action: Actionlike + Copy,
	Button: Component + InputManagerReference,
	Ev: Event + InputManagerReference,
>(
	action: Action,
	event_generator: impl Fn(Entity) -> Ev + Send + Sync + Clone + 'static,
) -> SystemConfigs {
	(
		fire_input_events::<Action, Ev>(action, event_generator.clone()),
		fire_button_events::<Button, Ev>(event_generator),
	)
		.into_configs()
}

pub trait InputManagerReference {
	fn input_manager(&self) -> Entity;
}

pub fn input_managers_where_action_fired<Ev: Event + InputManagerReference>(
) -> impl Fn(EventReader<Ev>) -> Vec<Entity> {
	move |mut ev_input: EventReader<Ev>| {
		ev_input
			.read()
			.map(InputManagerReference::input_manager)
			.collect()
	}
}
