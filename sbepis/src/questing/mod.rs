use std::fmt::{self, Display, Formatter};

use bevy::prelude::*;
use bevy::utils::HashMap;
use proposal::*;
use quest_markers::*;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use screen::*;
use uuid::Uuid;

use crate::entity::EntityKilled;
use crate::input::button_just_pressed;
use crate::inventory::{Inventory, Item};
use crate::iter_system::*;
use crate::menus::*;
use crate::npcs::Imp;
use crate::player_controller::{interact_with, PlayerAction};
use crate::util::map_event;
use crate::{gridbox_material, some_or_return, BoxBundle};

mod proposal;
mod quest_markers;
mod screen;

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
						.iter_flatten()
						.iter_inspect(propose_quest_if_none)
						.iter_inspect(complete_quest_if_done)
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
						.iter_inspect(close_menu)
						.iter_map(get_proposed_quest)
						.iter_flatten()
						.iter_inspect(add_quest_nodes)
						.iter_done(),
					input_managers_where_action_fired::<QuestDeclined>()
						.iter_inspect(close_menu)
						.iter_done(),
					get_ended_quests
						.iter_inspect(remove_quest)
						.iter_inspect(remove_quest_nodes)
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
