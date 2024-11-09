use crate::camera::PlayerCameraNode;
use crate::menus::MenuStack;
use crate::player_commands::note_holder::NoteNodeHolder;
use crate::player_commands::notes::PlayNoteAction;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::Actionlike;

#[derive(Component)]
pub struct CommandStaff;

// This should be enough information to map all notes
pub const F5_LINE_TOP: f32 = 15.0;
pub const STAFF_HEIGHT: f32 = 60.0;
pub const CLEF_HEIGHT: f32 = 80.0;
pub const LINE_HEIGHT: f32 = 2.0;

pub const QUARTER_NOTE_TOP_OFFSET: f32 = 41.0;
pub const QUARTER_NOTE_HEIGHT: f32 = 55.0;
pub const QUARTER_NOTE_LEFT_START: f32 = 40.0;
pub const QUARTER_NOTE_LEFT_SPACING: f32 = 20.0;

// Does top + height not actually equal bottom???
pub const QUARTER_NOTE_WEIRD_SPACING_OFFSET: f32 = 18.0;

pub fn spawn_staff(mut commands: Commands, asset_server: Res<AssetServer>) {
	let treble_clef = asset_server.load("treble_clef.png");

	// Background
	commands
		.spawn((
			Name::new("Staff"),
			NodeBundle {
				style: Style {
					width: Val::Percent(100.0),
					height: Val::Px(100.0),
					flex_direction: FlexDirection::Row,
					margin: UiRect::all(Val::Px(10.0)),
					padding: UiRect::axes(Val::Px(100.0), Val::Px(10.0)),
					display: Display::None,
					..default()
				},
				background_color: bevy::color::palettes::css::BEIGE.into(),
				..default()
			},
			CommandStaff,
			PlayerCameraNode,
		))
		.with_children(|parent| {
			// Clef
			parent.spawn((
				Name::new("Clef"),
				ImageBundle {
					image: treble_clef.into(),
					style: Style {
						position_type: PositionType::Absolute,
						height: Val::Px(CLEF_HEIGHT),
						..default()
					},
					..default()
				},
			));

			// Staff lines
			parent
				.spawn((
					Name::new("Staff lines"),
					NodeBundle {
						style: Style {
							flex_direction: FlexDirection::Column,
							flex_grow: 1.0,
							padding: UiRect::top(Val::Px(F5_LINE_TOP)),
							height: Val::Px(STAFF_HEIGHT),
							justify_content: JustifyContent::SpaceBetween,
							..default()
						},
						..default()
					},
					NoteNodeHolder::default(),
				))
				.with_children(|parent| {
					for i in 0..5 {
						parent.spawn((
							Name::new(format!("Line {i}")),
							NodeBundle {
								style: Style {
									width: Val::Percent(100.0),
									height: Val::Px(LINE_HEIGHT),
									..default()
								},
								background_color: Color::BLACK.into(),
								..default()
							},
						));
					}
				});
		});
}

#[derive(Actionlike, Clone, Copy, Eq, PartialEq, Hash, Reflect, Debug)]
pub enum ToggleStaffAction {
	ToggleStaff,
}

#[derive(Event, Default)]
pub struct ToggleStaffEvent;

#[derive(Resource, Default)]
pub struct StaffState {
	pub is_open: bool,
}

pub fn toggle_staff(mut staff_state: ResMut<StaffState>) {
	staff_state.is_open = !staff_state.is_open;
}

pub fn is_staff_open(staff_state: Res<StaffState>) -> bool {
	staff_state.is_open
}

pub fn show_staff(mut staff_style: Query<&mut Style, With<CommandStaff>>) {
	for mut style in staff_style.iter_mut() {
		style.display = Display::Flex;
	}
}

pub fn hide_staff(mut staff_style: Query<&mut Style, With<CommandStaff>>) {
	for mut style in staff_style.iter_mut() {
		style.display = Display::None;
	}
}

pub fn disable_note_input(
	mut menu_stack: ResMut<MenuStack>,
	note_input: Query<Entity, With<ActionState<PlayNoteAction>>>,
) {
	menu_stack.remove(note_input.single());
}

pub fn enable_note_input(
	mut menu_stack: ResMut<MenuStack>,
	note_input: Query<Entity, With<ActionState<PlayNoteAction>>>,
) {
	menu_stack.push(note_input.single());
}
