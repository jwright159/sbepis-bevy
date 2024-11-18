use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::iter_system::*;
use crate::player_controller::interact_with;

pub struct InventoryPlugin;
impl Plugin for InventoryPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			Update,
			(interact_with::<Item>
				.iter_filter_some()
				.iter_do(pick_up_items)
				.iter_done(),),
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

fn pick_up_items(
	In(item_entity): In<Entity>,
	mut commands: Commands,
	mut player: Query<&mut Inventory>,
) {
	let mut inventory = player.single_mut();
	inventory.items.push(item_entity);
	commands
		.entity(item_entity)
		.remove::<RigidBody>()
		.insert(Visibility::Hidden)
		.insert(ColliderDisabled);
}
