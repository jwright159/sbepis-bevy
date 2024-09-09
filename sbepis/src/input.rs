use std::any::type_name;

use bevy::{ecs::schedule::SystemConfigs, prelude::*};
use leafwing_input_manager::{plugin::InputManagerSystem, prelude::*};

pub fn spawn_input_manager<Action: Actionlike>(input_map: InputMap<Action>) -> SystemConfigs {
	(move |mut commands: Commands| {
		commands.spawn((
			Name::new(format!(
				"InputManager<{}>",
				type_name::<Action>().split("::").last().unwrap()
			)),
			InputManagerBundle::<Action> {
				input_map: input_map.clone(),
				..default()
			},
		));
	})
	.into_configs()
}

pub fn action_event<Action: Actionlike + Copy, EventType: Event>(
	event_generator: impl Fn(Action) -> EventType + Send + Sync + 'static,
) -> SystemConfigs {
	(move |input: Query<&ActionState<Action>>, mut event: EventWriter<EventType>| {
		let input = input.single();
		for action in input.get_just_pressed() {
			event.send(event_generator(action));
		}
	})
	.after(InputManagerSystem::ManualControl)
}

pub fn button_event<Action: Actionlike + Copy, EventType: Event>(
	action: Action,
	event_generator: impl Fn() -> EventType + Send + Sync + 'static,
) -> SystemConfigs {
	(move |input: Query<&ActionState<Action>>, mut event: EventWriter<EventType>| {
		let input = input.single();
		if input.just_pressed(&action) {
			event.send(event_generator());
		}
	})
	.after(InputManagerSystem::ManualControl)
}

pub fn button_input<Action: Actionlike + Copy>(
	action: Action,
) -> impl Fn(Query<&ActionState<Action>>) -> bool {
	move |input: Query<&ActionState<Action>>| {
		let input = input.single();
		input.pressed(&action)
	}
}

pub fn button_just_pressed<Action: Actionlike + Copy>(
	action: Action,
) -> impl Fn(Query<&ActionState<Action>>) -> bool {
	move |input: Query<&ActionState<Action>>| {
		let input = input.single();
		input.just_pressed(&action)
	}
}

pub fn dual_axes_input<Action: Actionlike + Copy>(
	action: Action,
) -> impl Fn(Query<&ActionState<Action>>) -> Vec2 {
	move |input: Query<&ActionState<Action>>| {
		let input = input.single();
		input.axis_pair(&action)
	}
}

pub fn clamped_dual_axes_input<Action: Actionlike + Copy>(
	action: Action,
) -> impl Fn(Query<&ActionState<Action>>) -> Vec2 {
	move |input: Query<&ActionState<Action>>| {
		let input = input.single();
		input.clamped_axis_pair(&action)
	}
}
