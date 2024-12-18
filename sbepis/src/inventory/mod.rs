use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use screen::*;

use crate::input::button_just_pressed;
use crate::iter_system::*;
use crate::menus::show_menu;
use crate::player_controller::{interact_with, PlayerAction};

mod screen;

pub struct InventoryPlugin;
impl Plugin for InventoryPlugin {
	fn build(&self, app: &mut App) {
		app.add_event::<ItemPickedUp>()
			.add_systems(Startup, spawn_inventory_screen)
			.add_systems(
				Update,
				(
					interact_with::<Item>
						.iter_flatten()
						.iter_inspect(pick_up_items)
						.iter_done(),
					show_menu::<InventoryScreen>
						.run_if(button_just_pressed(PlayerAction::OpenInventory)),
					add_item_to_inventory_screen,
				),
			);
	}
}

#[derive(Component, Default)]
pub struct Inventory {
	pub items: Vec<Entity>,
}

#[derive(Component)]
pub struct Item {
	pub icon: Handle<Image>,
}

#[derive(Event)]
pub struct ItemPickedUp(pub Entity);

fn pick_up_items(
	In(item_entity): In<Entity>,
	mut commands: Commands,
	mut player: Query<&mut Inventory>,
	mut ev_picked_up: EventWriter<ItemPickedUp>,
) {
	let mut inventory = player.single_mut();
	inventory.items.push(item_entity);
	commands
		.entity(item_entity)
		.remove::<RigidBody>()
		.insert(Visibility::Hidden)
		.insert(ColliderDisabled);
	ev_picked_up.send(ItemPickedUp(item_entity));
}
