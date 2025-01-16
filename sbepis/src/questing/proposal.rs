use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::dialogue::spawn_dialogue;
use crate::menus::*;

use super::{InputManagerReference, Quest, QuestGiver, QuestId, Quests};

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

pub fn propose_quest_if_none(
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

	let mut dialogue = spawn_dialogue(
		&mut commands,
		&mut menu_stack,
		format!("{}\n\n{}", quest.name, quest.description),
		QuestProposal { quest_id },
		InputMap::default()
			.with(QuestProposalAction::Accept, KeyCode::KeyE)
			.with(QuestProposalAction::Decline, KeyCode::Space),
	);
	dialogue.add_option(
		&mut commands,
		"Accept [E]".to_owned(),
		QuestProposalAccept {
			quest_proposal: dialogue.root,
		},
	);
	dialogue.add_option(
		&mut commands,
		"Decline [Space]".to_owned(),
		QuestProposalDecline {
			quest_proposal: dialogue.root,
		},
	);
}

pub fn get_proposed_quest(
	In(input): In<Entity>,
	quest_proposals: Query<&QuestProposal>,
) -> Option<QuestId> {
	quest_proposals.get(input).map(|qp| qp.quest_id).ok()
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
