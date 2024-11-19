use bevy::color::palettes::css;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::camera::PlayerCameraNode;
use crate::input::input_manager_bundle;
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
