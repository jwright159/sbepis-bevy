use std::f32::consts::PI;
use std::time::Duration;

use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;
use bevy::scene::SceneInstanceReady;
use bevy_butler::*;
use bevy_common_assets::ron::RonAssetPlugin;
use bevy_rapier3d::geometry::Collider;
use name_tags::*;

use crate::entity::spawner::{
	EntitySpawned, EntitySpawnedSet, SpawnerActivated, SpawnerActivatedSet,
};
use crate::entity::{
	Healing, Movement, RandomInput, RotateTowardMovement, SpawnHealthBar, TargetPlayer,
};
use crate::gridbox_material;
use crate::main_bundles::Mob;
use crate::questing::{QuestGiver, SpawnQuestMarker};

mod name_tags;

#[butler_plugin(build(
	add_plugins(RonAssetPlugin::<AvailableNames>::new(&["names.ron"])),
))]
pub struct NpcPlugin;

#[derive(Component)]
pub struct AnimationRootReference(pub Entity);

#[derive(Component)]
pub struct Consort;

#[derive(Component)]
pub struct ConsortSpawner;

#[system(
	plugin = NpcPlugin, schedule = Update,
	after = SpawnerActivatedSet,
	in_set = EntitySpawnedSet,
)]
fn spawn_consort(
	mut ev_spawner: EventReader<SpawnerActivated>,
	mut ev_spawned: EventWriter<EntitySpawned>,
	spawners: Query<(), With<ConsortSpawner>>,
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	asset_server: Res<AssetServer>,
) {
	for ev in ev_spawner.read() {
		if spawners.get(ev.spawner).is_err() {
			continue;
		}

		commands
			.entity(ev.entity)
			.insert((
				Name::new("Consort"),
				Transform::from_translation(ev.position),
				Mob,
				SpawnHealthBar,
				RandomInput::default(),
				Healing(0.2),
				RotateTowardMovement,
				Consort,
				QuestGiver::default(),
				SpawnQuestMarker,
				SpawnNameTag,
			))
			.with_child((
				Transform::from_translation(Vec3::Y * 0.5),
				Mesh3d(
					meshes.add(
						Capsule3d::new(0.25, 0.5)
							.mesh()
							.rings(1)
							.latitudes(8)
							.longitudes(16)
							.uv_profile(CapsuleUvProfile::Fixed),
					),
				),
				MeshMaterial3d(gridbox_material("magenta", &mut materials, &asset_server)),
				Collider::capsule_y(0.25, 0.25),
			));
		ev_spawned.send(EntitySpawned(ev.entity));
	}
}

#[derive(Component)]
pub struct Imp;

#[derive(Component)]
pub struct ImpSpawner;

#[derive(Component)]
pub struct InsertImpAssets;

#[derive(Resource)]
pub struct ImpAssets(pub Handle<Gltf>);

#[derive(Component)]
pub struct ImpAnimations {
	pub idle: AnimationNodeIndex,
	pub run: AnimationNodeIndex,
	pub attack: AnimationNodeIndex,
}

#[system(
	plugin = NpcPlugin, schedule = Startup,
)]
fn setup_imp_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.insert_resource(ImpAssets(asset_server.load("imp.glb")));
}

#[system(
	plugin = NpcPlugin, schedule = Update,
	after = SpawnerActivatedSet,
)]
fn queue_spawning_imp(
	mut ev_spawner: EventReader<SpawnerActivated>,
	mut commands: Commands,
	spawners: Query<(), With<ImpSpawner>>,
) {
	for ev in ev_spawner.read() {
		if spawners.get(ev.spawner).is_err() {
			continue;
		}

		commands.entity(ev.entity).insert((
			Name::new("Imp"),
			Transform::from_translation(ev.position),
			InsertImpAssets,
		));
	}
}

#[system(
	plugin = NpcPlugin, schedule = Update,
	after = queue_spawning_imp,
	in_set = EntitySpawnedSet,
)]
fn spawn_imp(
	imps: Query<Entity, With<InsertImpAssets>>,
	mut ev_spawned: EventWriter<EntitySpawned>,
	mut commands: Commands,
	imp_assets: Res<ImpAssets>,
	gltfs: Res<Assets<Gltf>>,
) {
	let Some(imp_gltf) = gltfs.get(&imp_assets.0) else {
		return;
	};

	for imp in imps.iter() {
		commands
			.entity(imp)
			.insert((
				SceneRoot(imp_gltf.scenes[0].clone()),
				Mob,
				SpawnHealthBar,
				TargetPlayer,
				RotateTowardMovement,
				Imp,
				SpawnNameTag,
			))
			.remove::<InsertImpAssets>()
			.with_child((
				Transform::from_translation(Vec3::Y * 0.5),
				Collider::capsule_y(0.25, 0.25),
			))
			.observe(
				|trigger: Trigger<SceneInstanceReady>,
				 mut commands: Commands,
				 imp_assets: Res<ImpAssets>,
				 gltfs: Res<Assets<Gltf>>,
				 mut animation_graphs: ResMut<Assets<AnimationGraph>>,
				 children: Query<&Children>| {
					let imp_gltf = gltfs
						.get(&imp_assets.0)
						.expect("Gltf should be loaded by now");

					let (animation_graph, nodes) = AnimationGraph::from_clips([
						imp_gltf.named_animations["Idle"].clone(),
						imp_gltf.named_animations["Run"].clone(),
						imp_gltf.named_animations["Attack"].clone(),
					]);
					let animation_graph = animation_graphs.add(animation_graph);

					let imp_animations = ImpAnimations {
						idle: nodes[0],
						run: nodes[1],
						attack: nodes[2],
					};

					let mut animation_player = AnimationPlayer::default();

					let mut transitions = AnimationTransitions::new();
					transitions.play(&mut animation_player, imp_animations.idle, Duration::ZERO);

					let scene = *children.get(trigger.entity()).unwrap().last().unwrap();
					let armature = children.get(scene).unwrap()[0];

					commands.entity(armature).insert((
						Transform::from_rotation(Quat::from_rotation_y(PI)),
						AnimationGraphHandle(animation_graph),
						transitions,
						animation_player,
						imp_animations,
					));

					commands
						.entity(trigger.entity())
						.insert(AnimationRootReference(armature));
				},
			);

		ev_spawned.send(EntitySpawned(imp));
	}
}

#[system(
	plugin = NpcPlugin, schedule = Update,
	after = EntitySpawnedSet,
)]
fn update_imp_animations(
	mut imps: Query<(&Movement, &AnimationRootReference), With<Imp>>,
	mut animations: Query<(
		&mut AnimationPlayer,
		&mut AnimationTransitions,
		&ImpAnimations,
	)>,
) {
	for (movement, scene_root) in imps.iter_mut() {
		let (mut animation_player, mut transitions, animations) =
			animations.get_mut(scene_root.0).unwrap();

		if movement.0.length() > 0.0 {
			if transitions
				.get_main_animation()
				.map(|index| index != animations.run)
				.unwrap_or(true)
			{
				transitions
					.play(
						&mut animation_player,
						animations.run,
						Duration::from_secs_f32(0.5),
					)
					.repeat();
			}
		} else if transitions
			.get_main_animation()
			.map(|index| index != animations.idle)
			.unwrap_or(true)
		{
			transitions
				.play(
					&mut animation_player,
					animations.idle,
					Duration::from_secs_f32(0.5),
				)
				.repeat();
		}
	}
}
