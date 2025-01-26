use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy_butler::*;
use leafwing_input_manager::prelude::InputMap;

use crate::camera::PlayerCameraNode;
use crate::input::input_manager_bundle;
use crate::inventory::{pick_up_items, InventoryPlugin, Item, ItemPickedUp};
use crate::menus::*;

#[derive(Component)]
pub struct InventoryScreen;

#[system(
	plugin = InventoryPlugin, schedule = Startup,
)]
fn spawn_inventory_screen(mut commands: Commands) {
	commands
		.spawn((
			Node {
				width: Val::Percent(100.0),
				height: Val::Percent(100.0),
				margin: UiRect::all(Val::Px(10.0)),
				row_gap: Val::Px(10.0),
				column_gap: Val::Px(10.0),
				flex_wrap: FlexWrap::Wrap,
				..default()
			},
			BackgroundColor(css::GRAY.with_alpha(0.5).into()),
			Visibility::Hidden,
			input_manager_bundle(
				InputMap::default().with(CloseMenuAction, KeyCode::KeyV),
				false,
			),
			PlayerCameraNode,
			Menu,
			MenuWithMouse,
			MenuWithInputManager,
			MenuHidesWhenClosed,
			InventoryScreen,
		))
		.insert(Name::new("Inventory Screen"));
}

#[system(
	plugin = InventoryPlugin, schedule = Update,
	after = pick_up_items,
)]
fn add_item_to_inventory_screen(
	mut ev_picked_up: EventReader<ItemPickedUp>,
	mut commands: Commands,
	items: Query<&Item>,
	inventory_screen: Query<Entity, With<InventoryScreen>>,
) {
	let inventory_screen = inventory_screen.single();

	for ItemPickedUp(item_entity) in ev_picked_up.read() {
		let item = items.get(*item_entity).expect("Item not found");

		commands
			.spawn((
				ImageNode::new(item.icon.clone()),
				Node {
					width: Val::Px(100.0),
					height: Val::Px(100.0),
					..default()
				},
				BackgroundColor(css::DARK_GRAY.into()),
			))
			.set_parent(inventory_screen);
	}
}
