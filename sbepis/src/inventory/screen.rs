use bevy::color::palettes::css;
use bevy::prelude::*;
use leafwing_input_manager::prelude::InputMap;

use crate::camera::PlayerCameraNode;
use crate::input::input_manager_bundle;
use crate::menus::*;

use super::{Item, ItemPickedUp};

#[derive(Component)]
pub struct InventoryScreen;

pub fn spawn_inventory_screen(mut commands: Commands) {
	commands
		.spawn((
			NodeBundle {
				style: Style {
					width: Val::Percent(100.0),
					height: Val::Percent(100.0),
					margin: UiRect::all(Val::Px(10.0)),
					row_gap: Val::Px(10.0),
					column_gap: Val::Px(10.0),
					flex_wrap: FlexWrap::Wrap,
					..default()
				},
				background_color: bevy::color::palettes::css::GRAY.with_alpha(0.5).into(),
				visibility: Visibility::Hidden,
				..default()
			},
			input_manager_bundle(
				InputMap::default().with(MenuAction::CloseMenu, KeyCode::KeyV),
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

pub fn add_item_to_inventory_screen(
	mut ev_picked_up: EventReader<ItemPickedUp>,
	mut commands: Commands,
	items: Query<&Item>,
	inventory_screen: Query<Entity, With<InventoryScreen>>,
) {
	let inventory_screen = inventory_screen.single();

	for ItemPickedUp(item_entity) in ev_picked_up.read() {
		let item = items.get(*item_entity).expect("Item not found");

		commands
			.spawn(ImageBundle {
				image: item.icon.clone().into(),
				style: Style {
					width: Val::Px(100.0),
					height: Val::Px(100.0),
					..default()
				},
				background_color: css::DARK_GRAY.into(),
				..default()
			})
			.set_parent(inventory_screen);
	}
}
