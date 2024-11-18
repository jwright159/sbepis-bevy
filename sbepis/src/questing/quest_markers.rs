use bevy::prelude::*;

use crate::{some_or_continue, some_or_return};

use super::{QuestGiver, Quests};

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

pub fn load_quest_markers(mut commands: Commands, asset_server: Res<AssetServer>) {
	let asset = asset_server.load("quest markers.glb");
	commands.insert_resource(QuestMarkerAsset(asset));
}

pub fn spawn_quest_markers(
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
			.spawn(SceneBundle {
				scene: asset.named_scenes["New"].clone(),
				visibility: Visibility::Visible,
				..default()
			})
			.id();

		let updated_marker = commands
			.spawn(SceneBundle {
				scene: asset.named_scenes["Updated"].clone(),
				visibility: Visibility::Hidden,
				..default()
			})
			.id();

		let marker = commands
			.spawn((
				Name::new("Quest Marker"),
				SpatialBundle::from_transform(Transform::from_xyz(0.0, 0.6, 0.0)),
				QuestMarker {
					entity: quest_giver_entity,
					new_marker,
					updated_marker,
				},
			))
			.set_parent(quest_giver_entity)
			.push_children(&[new_marker, updated_marker])
			.id();

		quest_giver.quest_marker = Some(marker);
	}
}

pub fn despawn_invalid_quest_markers(
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

pub fn update_quest_markers(
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
		let quest_marker = quest_markers
			.get(quest_marker)
			.expect("Quest marker not found");
		let [mut new_visibility, mut updated_visibility] =
			visibilities.many_mut([quest_marker.new_marker, quest_marker.updated_marker]);

		if let Some(quest_id) = quest_giver.given_quest {
			let quest = quests.0.get(&quest_id).expect("Quest not found");
			*new_visibility = Visibility::Hidden;
			*updated_visibility = if quest.completed {
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
