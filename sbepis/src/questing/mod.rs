use std::fmt::{self, Display, Formatter};

use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;
use quest_markers::*;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use uuid::Uuid;

use crate::camera::{PlayerCamera, PlayerCameraNode};
use crate::input::{button_just_pressed, input_manager_bundle};
use crate::iter_system::{
	DoSystemTrait, DoneSystemTrait, FilterOkSystemTrait, IteratorSystemTrait,
};
use crate::menus::*;
use crate::player_controller::PlayerAction;
use crate::some_or_return;

mod quest_markers;

pub use quest_markers::SpawnQuestMarker;

pub struct QuestingPlugin;
impl Plugin for QuestingPlugin {
	fn build(&self, app: &mut App) {
		app.register_type::<Quests>()
			.register_type::<QuestGiver>()
			.register_type::<QuestId>()
			.register_type::<Quest>()
			.init_resource::<Quests>()
			.add_event::<QuestAccepted>()
			.add_event::<QuestDeclined>()
			.add_plugins(InputManagerMenuPlugin::<QuestProposalAction>::default())
			.add_systems(Startup, (spawn_quest_screen, load_quest_markers))
			.add_systems(
				Update,
				(
					interact_with_quest_giver
						.pipe(propose_quest)
						.run_if(button_just_pressed(PlayerAction::Interact)),
					fire_input_and_button_events::<
						QuestProposalAction,
						QuestProposalAccept,
						QuestAccepted,
					>(QuestProposalAction::Accept, QuestAccepted::new),
					fire_input_and_button_events::<
						QuestProposalAction,
						QuestProposalDecline,
						QuestDeclined,
					>(QuestProposalAction::Decline, QuestDeclined::new),
					input_managers_where_action_fired::<QuestAccepted>()
						.iter_do(close_menu)
						.iter_map(get_proposed_quest)
						.iter_filter_some()
						.iter_do(add_quest_nodes)
						.iter_done(),
					input_managers_where_action_fired::<QuestDeclined>()
						.iter_do(close_menu)
						.iter_map(get_proposed_quest)
						.iter_filter_some()
						.iter_do(remove_quest)
						.iter_do(remove_quest_nodes)
						.iter_done(),
					change_displayed_node,
					show_menu::<QuestScreen>
						.run_if(button_just_pressed(PlayerAction::OpenQuestScreen)),
					spawn_quest_markers,
					despawn_invalid_quest_markers,
					update_quest_markers,
				),
			);

		#[cfg(feature = "inspector")]
		app.register_type_data::<QuestId, bevy_inspector_egui::inspector_egui_impls::InspectorEguiImpl>();
	}
}

#[derive(Resource, Default, Debug, Reflect)]
#[reflect(Resource)]
pub struct Quests(pub HashMap<QuestId, Quest>);

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Reflect)]
pub struct QuestId(Uuid);
impl QuestId {
	#[allow(clippy::new_without_default)]
	pub fn new() -> Self {
		Self(Uuid::new_v4())
	}
}
impl Display for QuestId {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}
#[cfg(feature = "inspector")]
impl bevy_inspector_egui::inspector_egui_impls::InspectorPrimitive for QuestId {
	fn ui(
		&mut self,
		ui: &mut bevy_inspector_egui::egui::Ui,
		_options: &dyn std::any::Any,
		_id: bevy_inspector_egui::egui::Id,
		_env: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
	) -> bool {
		ui.add_enabled_ui(false, |ui| {
			ui.text_edit_singleline(&mut self.0.to_string());
		});
		false
	}

	fn ui_readonly(
		&self,
		ui: &mut bevy_inspector_egui::egui::Ui,
		_options: &dyn std::any::Any,
		_id: bevy_inspector_egui::egui::Id,
		_env: bevy_inspector_egui::reflect_inspector::InspectorUi<'_, '_>,
	) {
		ui.add_enabled_ui(false, |ui| {
			ui.text_edit_singleline(&mut self.0.to_string());
		});
	}
}

#[derive(Debug, Reflect)]
pub enum QuestType {
	Fetch,
	Kill(u32),
}
impl Distribution<QuestType> for Standard {
	fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> QuestType {
		match rng.gen_range(0..=1) {
			0 => QuestType::Fetch,
			_ => QuestType::Kill(rng.gen_range(1..=5)),
		}
	}
}

#[derive(Debug, Reflect)]
pub struct Quest {
	pub id: QuestId,
	pub quest_type: QuestType,
	pub name: String,
	pub description: String,
	pub completed: bool,
}
impl Distribution<Quest> for Standard {
	fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Quest {
		let quest_type: QuestType = rng.gen();
		match quest_type {
			QuestType::Kill(amount) => Quest {
				id: QuestId::new(),
				name: "Awesome Kill Quest".to_string(),
				quest_type,
				description: format!("imps killed my grandma... pwease go take revenge on those darn imps for me... kill {amount}!!"),
				completed: false,
			},
			QuestType::Fetch => Quest {
				id: QuestId::new(),
				name: "Awesome Fetch Quest".to_string(),
				quest_type,
				description: "imps stole my orange cube... pwease go get it back!!".to_string(),
				completed: false,
			},
		}
	}
}

#[derive(Component, Default, Reflect)]
pub struct QuestGiver {
	pub given_quest: Option<QuestId>,
	quest_marker: Option<Entity>,
}

#[derive(Component)]
pub struct QuestScreen;

#[derive(Component)]
pub struct QuestScreenNodeList;

#[derive(Component)]
pub struct QuestScreenNodeDisplay(Option<Entity>);

#[derive(Component)]
pub struct QuestScreenNode {
	pub quest_id: QuestId,
	pub display: Entity,
}

#[derive(Component)]
pub struct QuestProposal {
	pub quest_id: QuestId,
}

#[derive(Component)]
pub struct QuestProposalAccept {
	pub quest_proposal: Entity,
}
impl InputManagerReference for QuestProposalAccept {
	fn input_manager(&self) -> Entity {
		self.quest_proposal
	}
}

#[derive(Component)]
pub struct QuestProposalDecline {
	pub quest_proposal: Entity,
}
impl InputManagerReference for QuestProposalDecline {
	fn input_manager(&self) -> Entity {
		self.quest_proposal
	}
}

#[derive(Event)]
pub struct QuestAccepted {
	pub quest_proposal: Entity,
}
impl QuestAccepted {
	pub fn new(quest_proposal: Entity) -> Self {
		Self { quest_proposal }
	}
}
impl InputManagerReference for QuestAccepted {
	fn input_manager(&self) -> Entity {
		self.quest_proposal
	}
}

#[derive(Event)]
pub struct QuestDeclined {
	pub quest_proposal: Entity,
}
impl QuestDeclined {
	pub fn new(quest_proposal: Entity) -> Self {
		Self { quest_proposal }
	}
}
impl InputManagerReference for QuestDeclined {
	fn input_manager(&self) -> Entity {
		self.quest_proposal
	}
}

fn spawn_quest_screen(mut commands: Commands) {
	commands
		.spawn((
			NodeBundle {
				style: Style {
					width: Val::Percent(100.0),
					height: Val::Percent(100.0),
					..default()
				},
				background_color: bevy::color::palettes::css::GRAY.with_alpha(0.5).into(),
				visibility: Visibility::Hidden,
				..default()
			},
			input_manager_bundle(
				InputMap::default().with(MenuAction::CloseMenu, KeyCode::KeyJ),
				false,
			),
			PlayerCameraNode,
			Menu,
			MenuWithMouse,
			MenuWithInputManager,
			MenuHidesWhenClosed,
			QuestScreen,
		))
		.insert(Name::new("Quest Screen"))
		.with_children(|parent| {
			parent.spawn((
				NodeBundle {
					style: Style {
						flex_grow: 1.0,
						flex_direction: FlexDirection::Column,
						..default()
					},
					..default()
				},
				QuestScreenNodeList,
			));
			parent.spawn((NodeBundle {
				style: Style {
					width: Val::Px(2.0),
					..default()
				},
				background_color: css::WHITE.into(),
				..default()
			},));
			parent.spawn((
				NodeBundle {
					style: Style {
						flex_grow: 4.0,
						..default()
					},
					..default()
				},
				QuestScreenNodeDisplay(None),
			));
		});
}

fn interact_with_quest_giver(
	rapier_context: Res<RapierContext>,
	player_camera: Query<&GlobalTransform, With<PlayerCamera>>,
	quest_givers: Query<Entity, With<QuestGiver>>,
) -> Option<Entity> {
	let player_camera = player_camera.get_single().expect("Player camera missing");
	let mut quest_giver = None;
	rapier_context.intersections_with_ray(
		player_camera.translation(),
		player_camera.forward().into(),
		3.0,
		false,
		QueryFilter::default(),
		|entity, _intersection| {
			if quest_givers.get(entity).is_ok() {
				quest_giver = Some(entity);
				false
			} else {
				true
			}
		},
	);
	quest_giver
}

fn propose_quest(
	In(quest_giver): In<Option<Entity>>,
	mut commands: Commands,
	mut quests: ResMut<Quests>,
	mut quest_givers: Query<&mut QuestGiver>,
	mut menu_stack: ResMut<MenuStack>,
) {
	let quest_giver = some_or_return!(quest_giver);
	let mut quest_giver = quest_givers
		.get_mut(quest_giver)
		.expect("Quest giver missing");
	if quest_giver.given_quest.is_some() {
		return;
	}

	let quest: Quest = rand::random();
	let quest_id = quest.id;
	quests.0.insert(quest_id, quest);
	let quest = quests
		.0
		.get(&quest_id)
		.expect("Unknown quest even though we just inserted it");

	quest_giver.given_quest = Some(quest_id);

	let proposal = commands
		.spawn((
			NodeBundle {
				style: Style {
					margin: UiRect::all(Val::Auto),
					width: Val::Percent(100.0),
					max_width: Val::Px(600.0),
					padding: UiRect::all(Val::Px(10.0)),
					flex_direction: FlexDirection::Column,
					..default()
				},
				background_color: css::GRAY.into(),
				..default()
			},
			PlayerCameraNode,
			input_manager_bundle(
				InputMap::default()
					.with(QuestProposalAction::Accept, KeyCode::KeyE)
					.with(QuestProposalAction::Decline, KeyCode::Space),
				false,
			),
			Menu,
			MenuWithMouse,
			MenuWithInputManager,
			MenuDespawnsWhenClosed,
			QuestProposal { quest_id },
		))
		.insert(Name::new(format!("Quest Proposal for {quest_id}")))
		.with_children(|parent| {
			let proposal = parent.parent_entity();

			parent.spawn(TextBundle {
				text: Text::from_section(
					format!("{}\n\n{}", quest.name, quest.description),
					TextStyle {
						font_size: 20.0,
						color: Color::WHITE,
						..default()
					},
				),
				style: Style {
					margin: UiRect::bottom(Val::Px(10.0)),
					..default()
				},
				..default()
			});
			parent
				.spawn(NodeBundle {
					style: Style {
						flex_direction: FlexDirection::Row,
						column_gap: Val::Px(10.0),
						..default()
					},
					..default()
				})
				.with_children(|parent| {
					parent
						.spawn((
							ButtonBundle {
								style: Style {
									padding: UiRect::all(Val::Px(10.0)),
									flex_grow: 1.0,
									..default()
								},
								background_color: css::DARK_GRAY.into(),
								..default()
							},
							QuestProposalAccept {
								quest_proposal: proposal,
							},
						))
						.with_children(|parent| {
							parent.spawn(TextBundle {
								text: Text::from_section(
									"Accept [E]",
									TextStyle {
										font_size: 20.0,
										color: Color::WHITE,
										..default()
									},
								),
								..default()
							});
						});
					parent
						.spawn((
							ButtonBundle {
								style: Style {
									padding: UiRect::all(Val::Px(10.0)),
									flex_grow: 1.0,
									..default()
								},
								background_color: css::DARK_GRAY.into(),
								..default()
							},
							QuestProposalDecline {
								quest_proposal: proposal,
							},
						))
						.with_children(|parent| {
							parent.spawn(TextBundle {
								text: Text::from_section(
									"Decline [Space]",
									TextStyle {
										font_size: 20.0,
										color: Color::WHITE,
										..default()
									},
								),
								..default()
							});
						});
				});
		})
		.id();

	menu_stack.push(proposal);
}

fn get_proposed_quest(
	In(input): In<Entity>,
	quest_proposals: Query<&QuestProposal>,
) -> Option<QuestId> {
	quest_proposals.get(input).map(|qp| qp.quest_id).ok()
}

fn remove_quest(
	In(quest_id): In<QuestId>,
	mut quests: ResMut<Quests>,
	mut quest_givers: Query<&mut QuestGiver>,
) {
	quests.0.remove(&quest_id);

	let mut quest_giver = quest_givers
		.iter_mut()
		.find(|qg| qg.given_quest == Some(quest_id))
		.expect("Quest giver missing");
	quest_giver.given_quest = None;
}

fn add_quest_nodes(
	In(quest_id): In<QuestId>,
	mut commands: Commands,
	quests: Res<Quests>,
	quest_screen_node_list: Query<Entity, With<QuestScreenNodeList>>,
	quest_screen_node_display: Query<Entity, With<QuestScreenNodeDisplay>>,
) {
	let quest_screen_node_list = quest_screen_node_list.single();
	let quest_screen_node_display = quest_screen_node_display.single();

	let quest = quests.0.get(&quest_id).expect("Unknown quest");

	let display = commands
		.spawn(TextBundle {
			text: Text::from_section(
				quest.description.clone(),
				TextStyle {
					font_size: 20.0,
					color: Color::WHITE,
					..default()
				},
			),
			style: Style {
				display: bevy::ui::Display::None,
				..default()
			},
			..default()
		})
		.set_parent(quest_screen_node_display)
		.id();

	commands
		.spawn((
			ButtonBundle {
				style: Style {
					padding: UiRect::all(Val::Px(10.0)),
					width: Val::Percent(100.0),
					..default()
				},
				background_color: css::GRAY.into(),
				..default()
			},
			QuestScreenNode { quest_id, display },
		))
		.set_parent(quest_screen_node_list)
		.with_children(|parent| {
			parent.spawn(TextBundle {
				text: Text::from_section(
					quest.name.clone(),
					TextStyle {
						font_size: 20.0,
						color: Color::WHITE,
						..default()
					},
				),
				..default()
			});
		});
}

fn remove_quest_nodes(
	In(quest_id): In<QuestId>,
	mut commands: Commands,
	quest_nodes: Query<(Entity, &QuestScreenNode)>,
) {
	for (quest_node_entity, quest_node) in quest_nodes
		.iter()
		.filter(|(_, node)| node.quest_id == quest_id)
	{
		commands.entity(quest_node_entity).despawn_recursive();
		commands.entity(quest_node.display).despawn_recursive();
	}
}

fn change_displayed_node(
	quest_nodes: Query<(&QuestScreenNode, &Interaction), Changed<Interaction>>,
	mut quest_node_displays: Query<&mut Style>,
	mut quest_screen_node_display: Query<&mut QuestScreenNodeDisplay>,
) {
	let mut quest_screen_node_display = quest_screen_node_display.single_mut();

	for (quest_node, &interaction) in quest_nodes.iter() {
		if interaction == Interaction::Pressed {
			if let Some(mut style) = quest_screen_node_display
				.0
				.and_then(|e| quest_node_displays.get_mut(e).ok())
			{
				style.display = bevy::ui::Display::None;
			}

			if let Ok(mut style) = quest_node_displays.get_mut(quest_node.display) {
				style.display = bevy::ui::Display::DEFAULT;
				quest_screen_node_display.0 = Some(quest_node.display);
			}
		}
	}
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Reflect, Debug)]
pub enum QuestProposalAction {
	Accept,
	Decline,
}
impl Actionlike for QuestProposalAction {
	fn input_control_kind(&self) -> InputControlKind {
		match self {
			QuestProposalAction::Accept => InputControlKind::Button,
			QuestProposalAction::Decline => InputControlKind::Button,
		}
	}
}
