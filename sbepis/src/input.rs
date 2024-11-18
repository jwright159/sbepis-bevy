use std::any::type_name;

use bevy::{ecs::schedule::SystemConfigs, prelude::*};
use leafwing_input_manager::{plugin::InputManagerSystem, prelude::*};

pub fn spawn_input_manager<Action: Actionlike>(
	input_map: InputMap<Action>,
	start_enabled: bool,
) -> SystemConfigs {
	(move |mut commands: Commands| {
		commands.spawn(input_manager_bundle(input_map.clone(), start_enabled));
	})
	.into_configs()
}

pub fn input_manager_bundle<Action: Actionlike>(
	input_map: InputMap<Action>,
	start_enabled: bool,
) -> impl Bundle {
	let mut action_state: ActionState<Action> = default();
	if !start_enabled {
		action_state.disable();
	}

	(
		Name::new(format!(
			"InputManager<{}>",
			type_name::<Action>().split("::").last().unwrap()
		)),
		InputManagerBundle::<Action> {
			input_map,
			action_state,
		},
	)
}

pub fn action_event<Action: Actionlike + Copy, EventType: Event>(
	event_generator: impl Fn(Action) -> EventType + Send + Sync + 'static,
) -> SystemConfigs {
	(move |input: Query<&ActionState<Action>>, mut event: EventWriter<EventType>| {
		for input in input.iter().filter(|input| !input.disabled()) {
			for action in input.get_just_pressed() {
				event.send(event_generator(action));
			}
		}
	})
	.after(InputManagerSystem::ManualControl)
}

pub fn button_event<Action: Actionlike + Copy, EventType: Event>(
	action: Action,
	event_generator: impl Fn() -> EventType + Send + Sync + 'static,
) -> SystemConfigs {
	(move |input: Query<&ActionState<Action>>, mut event: EventWriter<EventType>| {
		for input in input.iter().filter(|input| !input.disabled()) {
			if input.just_pressed(&action) {
				event.send(event_generator());
			}
		}
	})
	.after(InputManagerSystem::ManualControl)
}

pub fn input_managers_where_button_just_pressed<Action: Actionlike + Copy>(
	action: Action,
) -> impl Fn(Query<(Entity, &ActionState<Action>)>) -> Vec<Entity> {
	move |input: Query<(Entity, &ActionState<Action>)>| {
		input
			.iter()
			.filter(|(_, input)| input.just_pressed(&action))
			.map(|(entity, _)| entity)
			.collect()
	}
}

macro_rules! value_input {
	($function_name:ident, $value_type:ident, $default_value:expr => $default_value_type:ident) => {
		pub fn $function_name<Action: Actionlike + Copy>(
			action: Action,
		) -> impl Fn(Query<&ActionState<Action>>) -> $default_value_type {
			move |input: Query<&ActionState<Action>>| {
				if let Some(input) = input.iter().find(|input| !input.disabled()) {
					input.$value_type(&action)
				} else {
					$default_value
				}
			}
		}
	};
}

value_input!(button_input, pressed, false => bool);
value_input!(button_just_pressed, just_pressed, false => bool);
value_input!(axis_input, value, 0.0 => f32);
value_input!(clamped_axis_input, clamped_value, 0.0 => f32);
value_input!(dual_axes_input, axis_pair, Vec2::ZERO => Vec2);
value_input!(clamped_dual_axes_input, clamped_axis_pair, Vec2::ZERO => Vec2);
