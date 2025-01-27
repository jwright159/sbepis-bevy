use bevy::prelude::*;
use bevy_butler::*;

use crate::entity::EntityKilledSet;
use crate::questing::{QuestGiver, QuestingPlugin, Quests};
use crate::{some_or_continue, some_or_return};

#[derive(Component)]
pub struct SpawnQuestMarker;

#[derive(Component)]
pub struct QuestMarker {
	entity: Entity,
	new_marker: Entity,
	updated_marker: Entity,
}

#[derive(Resource)]
pub struct QuestMarkerAsset(Handle<Gltf>);

#[system(
	plugin = QuestingPlugin, schedule = Startup,
)]
fn load_quest_markers(mut commands: Commands, asset_server: Res<AssetServer>) {
	let asset = asset_server.load("quest markers.glb");
	commands.insert_resource(QuestMarkerAsset(asset));
}

#[system(
	plugin = QuestingPlugin, schedule = Update,
)]
fn spawn_quest_markers(
	mut commands: Commands,
	mut quest_givers: Query<(Entity, &mut QuestGiver), With<SpawnQuestMarker>>,
	asset: Res<QuestMarkerAsset>,
	assets: Res<Assets<Gltf>>,
) {
	let asset = some_or_return!(assets.get(&asset.0));

	for (quest_giver_entity, mut quest_giver) in quest_givers.iter_mut() {
		commands
			.entity(quest_giver_entity)
			.remove::<SpawnQuestMarker>();

		let new_marker = commands
			.spawn((
				SceneRoot(asset.named_scenes["New"].clone()),
				Visibility::Visible,
			))
			.id();

		let updated_marker = commands
			.spawn((
				SceneRoot(asset.named_scenes["Updated"].clone()),
				Visibility::Hidden,
			))
			.id();

		let marker = commands
			.spawn((
				Name::new("Quest Marker"),
				Transform::from_xyz(0.0, 2.0, 0.0),
				Visibility::Inherited,
				QuestMarker {
					entity: quest_giver_entity,
					new_marker,
					updated_marker,
				},
			))
			.set_parent(quest_giver_entity)
			.add_children(&[new_marker, updated_marker])
			.id();

		quest_giver.quest_marker = Some(marker);
	}
}

#[system(
	plugin = QuestingPlugin, schedule = Update,
	after = EntityKilledSet
)]
fn despawn_invalid_quest_markers(
	mut commands: Commands,
	quest_markers: Query<(Entity, &QuestMarker)>,
	entities: Query<Entity>,
) {
	for (quest_marker_entity, quest_marker) in quest_markers.iter() {
		if entities.get(quest_marker.entity).is_err() {
			commands.entity(quest_marker_entity).despawn_recursive();
		}
	}
}

#[system(
	plugin = QuestingPlugin, schedule = Update,
)]
fn update_quest_markers(
	quests: Res<Quests>,
	quest_givers: Query<&QuestGiver>,
	quest_markers: Query<&QuestMarker>,
	mut visibilities: Query<&mut Visibility>,
) {
	if !quests.is_changed() {
		return;
	}

	for quest_giver in quest_givers.iter() {
		let quest_marker = some_or_continue!(quest_giver.quest_marker);
		let quest_marker = some_or_continue!(quest_markers.get(quest_marker).ok()); // might still be loading
		let [mut new_visibility, mut updated_visibility] =
			visibilities.many_mut([quest_marker.new_marker, quest_marker.updated_marker]);

		if let Some(quest_id) = quest_giver.given_quest {
			let quest = quests.0.get(&quest_id).expect("Quest not found");
			*new_visibility = Visibility::Hidden;
			*updated_visibility = if quest.quest_type.is_completed() {
				Visibility::Visible
			} else {
				Visibility::Hidden
			};
		} else {
			*new_visibility = Visibility::Visible;
			*updated_visibility = Visibility::Hidden;
		}
	}
}
