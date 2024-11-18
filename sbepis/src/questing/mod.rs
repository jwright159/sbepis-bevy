use std::fmt::{self, Display, Formatter};

use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy::utils::HashMap;
use leafwing_input_manager::prelude::*;
use quest_markers::*;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use uuid::Uuid;

use crate::camera::PlayerCameraNode;
use crate::entity::EntityKilled;
use crate::input::{button_just_pressed, input_manager_bundle};
use crate::inventory::{Inventory, Item};
use crate::iter_system::*;
use crate::npcs::Imp;
use crate::player_controller::{interact_with, PlayerAction};
use crate::util::{map_event, MapRange};
use crate::{gridbox_material, menus::*, some_or_return, BoxBundle};

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
			.add_event::<QuestEnded>()
			.add_event::<QuestCompleted>()
			.add_plugins(InputManagerMenuPlugin::<QuestProposalAction>::default())
			.add_systems(Startup, (spawn_quest_screen, load_quest_markers))
			.add_systems(
				Update,
				(
					interact_with::<QuestGiver>
						.iter_filter_some()
						.iter_do(propose_quest_if_none)
						.iter_do(complete_quest_if_done)
						.iter_done(),
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
						.iter_done(),
					get_ended_quests
						.iter_do(remove_quest)
						.iter_do(remove_quest_nodes)
						.iter_done(),
					change_displayed_node,
					show_menu::<QuestScreen>
						.run_if(button_just_pressed(PlayerAction::OpenQuestScreen)),
					spawn_quest_markers,
					despawn_invalid_quest_markers,
					update_quest_node_progress,
					update_quest_markers,
					update_killed_imps,
					update_picked_up_items,
					map_event(|In(ev): In<QuestDeclined>, prop: Query<&QuestProposal>| {
						QuestEnded(prop.get(ev.quest_proposal).unwrap().quest_id)
					}),
					map_event(|In(ev): In<QuestCompleted>| QuestEnded(ev.0)),
					spawn_quest_drops,
					consume_quest_drop,
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
	Fetch { done: bool },
	Kill { amount: u32, done: u32 },
}
impl QuestType {
	pub fn is_completed(&self) -> bool {
		match self {
			QuestType::Fetch { done } => *done,
			QuestType::Kill { amount, done } => *done >= *amount,
		}
	}

	pub fn min_progress(&self) -> u32 {
		match self {
			QuestType::Fetch { .. } => 0,
			QuestType::Kill { .. } => 0,
		}
	}

	pub fn max_progress(&self) -> u32 {
		match self {
			QuestType::Fetch { .. } => 1,
			QuestType::Kill { amount, .. } => *amount,
		}
	}

	pub fn progress(&self) -> u32 {
		match self {
			QuestType::Fetch { done } => *done as u32,
			QuestType::Kill { done, amount } => (*done).min(*amount),
		}
	}

	pub fn progress_range(&self) -> std::ops::Range<f32> {
		self.min_progress() as f32..self.max_progress() as f32
	}
}
impl Distribution<QuestType> for Standard {
	fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> QuestType {
		match rng.gen_range(0..=1) {
			0 => QuestType::Fetch { done: false },
			_ => QuestType::Kill {
				amount: rng.gen_range(1..=5),
				done: 0,
			},
		}
	}
}

#[derive(Debug, Reflect)]
pub struct Quest {
	pub id: QuestId,
	pub quest_type: QuestType,
	pub name: String,
	pub description: String,
}
impl Distribution<Quest> for Standard {
	fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Quest {
		let quest_type: QuestType = rng.gen();
		match quest_type {
			QuestType::Kill {
				amount, ..
			} => Quest {
				id: QuestId::new(),
				name: "Awesome Kill Quest".to_string(),
				quest_type,
				description: format!("imps killed my grandma... pwease go take revenge on those darn imps for me... kill {amount}!!"),
			},
			QuestType::Fetch { .. } => Quest {
				id: QuestId::new(),
				name: "Awesome Fetch Quest".to_string(),
				quest_type,
				description: "imps stole my orange cube... pwease go get it back!!".to_string(),
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
	pub progress_text: Entity,
	pub progress_bar: Entity,
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

// ...These should probably all store QuestIds
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

#[derive(Event, Clone)]
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

#[derive(Event)]
pub struct QuestEnded(pub QuestId);

#[derive(Event, Clone)]
pub struct QuestCompleted(pub QuestId);

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

fn propose_quest_if_none(
	In(quest_giver): In<Entity>,
	mut commands: Commands,
	mut quests: ResMut<Quests>,
	mut quest_givers: Query<&mut QuestGiver>,
	mut menu_stack: ResMut<MenuStack>,
) {
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

fn complete_quest_if_done(
	In(entity): In<Entity>,
	mut ev_completed: EventWriter<QuestCompleted>,
	quests: Res<Quests>,
	quest_givers: Query<&QuestGiver>,
) {
	let quest_proposal = quest_givers.get(entity).expect("Quest giver not found");
	let quest_id = some_or_return!(quest_proposal.given_quest);
	let quest = quests.0.get(&quest_id).expect("Unknown quest");
	if !quest.quest_type.is_completed() {
		return;
	}
	ev_completed.send(QuestCompleted(quest_id));
}

fn get_ended_quests(mut ev_ended: EventReader<QuestEnded>) -> Vec<QuestId> {
	ev_ended.read().map(|ev| ev.0).collect()
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

	let mut progress_text: Option<Entity> = None;
	let mut progress_bar: Option<Entity> = None;

	let display = commands
		.spawn(NodeBundle {
			style: Style {
				display: bevy::ui::Display::None,
				flex_direction: FlexDirection::Column,
				..default()
			},
			..default()
		})
		.with_children(|parent| {
			parent.spawn(TextBundle {
				text: Text::from_section(
					quest.description.clone(),
					TextStyle {
						font_size: 20.0,
						color: Color::WHITE,
						..default()
					},
				),
				..default()
			});
			progress_text = Some(
				parent
					.spawn(TextBundle {
						text: Text::from_section(
							format!(
								"{}/{}",
								quest.quest_type.progress(),
								quest.quest_type.max_progress()
							),
							TextStyle {
								font_size: 20.0,
								color: Color::WHITE,
								..default()
							},
						),
						..default()
					})
					.id(),
			);
			parent
				.spawn(NodeBundle {
					style: Style {
						height: Val::Px(30.0),
						width: Val::Percent(100.0),
						..default()
					},
					background_color: css::DARK_GRAY.into(),
					..default()
				})
				.with_children(|parent| {
					progress_bar = Some(
						parent
							.spawn(NodeBundle {
								style: Style {
									width: Val::Percent(0.0),
									height: Val::Percent(100.0),
									..default()
								},
								background_color: css::LIGHT_GRAY.into(),
								..default()
							})
							.id(),
					);
				});
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
			QuestScreenNode {
				quest_id,
				display,
				progress_text: progress_text.unwrap(),
				progress_bar: progress_bar.unwrap(),
			},
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

fn update_quest_node_progress(
	quests: Res<Quests>,
	mut quest_nodes: Query<&QuestScreenNode>,
	mut progress_texts: Query<&mut Text>,
	mut progress_bars: Query<&mut Style>,
) {
	if !quests.is_changed() {
		return;
	}

	for quest_node in quest_nodes.iter_mut() {
		let quest = quests.0.get(&quest_node.quest_id).expect("Unknown quest");
		let mut progress_text = progress_texts.get_mut(quest_node.progress_text).unwrap();
		let mut progress_bar = progress_bars.get_mut(quest_node.progress_bar).unwrap();

		progress_text.sections[0].value = format!(
			"{}/{}",
			quest.quest_type.progress(),
			quest.quest_type.max_progress()
		);
		progress_bar.width = Val::Percent(
			(quest.quest_type.progress() as f32)
				.map_range(quest.quest_type.progress_range(), 0.0..100.0),
		);
	}
}

fn update_killed_imps(
	mut ev_killed: EventReader<EntityKilled>,
	mut quests: ResMut<Quests>,
	imps: Query<(), With<Imp>>,
) {
	for EntityKilled { entity } in ev_killed.read() {
		if imps.get(*entity).is_ok() {
			for (_, quest) in quests.0.iter_mut() {
				if let QuestType::Kill { done, .. } = &mut quest.quest_type {
					*done += 1;
				}
			}
		}
	}
}

fn update_picked_up_items(
	inventories: Query<&Inventory>,
	changed_inventories: Query<&Inventory, Changed<Inventory>>,
	mut quests: ResMut<Quests>,
) {
	let inventory = some_or_return!(if quests.is_changed() {
		inventories.iter().next()
	} else {
		changed_inventories.iter().next()
	});
	let num_items = inventory.items.len();
	for (_, quest) in quests.0.iter_mut() {
		if let QuestType::Fetch { done } = &mut quest.quest_type {
			*done = num_items > 0;
		}
	}
}

fn spawn_quest_drops(
	mut ev_killed: EventReader<EntityKilled>,
	mut commands: Commands,
	quests: Res<Quests>,
	imps: Query<&Transform, With<Imp>>,
	items: Query<&Item>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
) {
	let num_fetch_quests = quests
		.0
		.values()
		.filter(|quest| matches!(quest.quest_type, QuestType::Fetch { .. }))
		.count();
	let mut num_items = items.iter().count();

	for EntityKilled { entity } in ev_killed.read() {
		if num_items >= num_fetch_quests {
			break;
		}

		if let Ok(transform) = imps.get(*entity) {
			if rand::random() {
				continue;
			}

			commands.spawn((
				BoxBundle::new(
					transform.translation + Vec3::Y * 0.2,
					meshes.add(Cuboid::from_size(Vec3::splat(0.2))),
					gridbox_material("orange", &mut materials, &asset_server),
				)
				.with_collider_size(0.1),
				Item {
					icon: asset_server.load("item.png"),
				},
			));
			num_items += 1;
		}
	}
}

fn consume_quest_drop(
	mut ev_completed: EventReader<QuestCompleted>,
	mut inventories: Query<&mut Inventory>,
	mut commands: Commands,
	quests: Res<Quests>,
) {
	for QuestCompleted(quest_id) in ev_completed.read() {
		let quest = quests.0.get(quest_id).expect("Unknown quest");
		if let QuestType::Fetch { .. } = &quest.quest_type {
			if quest.quest_type.is_completed() {
				let mut inventory = inventories.single_mut();
				let item = inventory.items.pop().expect("No item to consume");
				commands.entity(item).despawn_recursive();
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
