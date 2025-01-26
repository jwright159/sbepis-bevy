use std::f32::consts::PI;

use bevy::animation::{animated_field, AnimationTarget, AnimationTargetId};
use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
use bevy::render::mesh::CapsuleUvProfile;

use crate::fray::FrayMusic;
use crate::gridbox_material;
use crate::player_controller::weapons::{DamageSweep, EndDamageSweep, SweepPivot, WeaponAnimation};

#[derive(Component)]
pub struct SwordPivot {
	pub blade: Entity,
}

#[derive(Component)]
pub struct Sword {
	pub damage: f32,
	pub wielder: Entity,
	pub allies: EntityHashSet,
	pub current_slash_damage: f32,
	pub current_slash_modifier: f32,
	side: SwordSide,
	pub left_swing_animation: AnimationNodeIndex,
	pub right_swing_animation: AnimationNodeIndex,
	pub woosh_sound: Handle<AudioSource>,
}

impl Sword {
	pub fn new(
		damage: f32,
		wielder: Entity,
		allies: EntityHashSet,
		left_swing_animation: AnimationNodeIndex,
		right_swing_animation: AnimationNodeIndex,
		woosh_sound: Handle<AudioSource>,
	) -> Self {
		Self {
			damage,
			wielder,
			allies,
			current_slash_damage: 0.0,
			current_slash_modifier: 0.0,
			side: SwordSide::Left,
			left_swing_animation,
			right_swing_animation,
			woosh_sound,
		}
	}
}

enum SwordSide {
	Left,
	Right,
}

impl SwordSide {
	fn other_side(&self) -> Self {
		match self {
			SwordSide::Left => SwordSide::Right,
			SwordSide::Right => SwordSide::Left,
		}
	}

	fn angle(&self) -> f32 {
		match self {
			SwordSide::Left => -PI * 0.5,
			SwordSide::Right => PI * 0.5,
		}
	}
}

pub fn spawn_sword(
	commands: &mut Commands,
	asset_server: &AssetServer,
	materials: &mut Assets<StandardMaterial>,
	meshes: &mut Assets<Mesh>,
	animations: &mut Assets<AnimationClip>,
	graphs: &mut Assets<AnimationGraph>,
	body: Entity,
) -> (Entity, Entity) {
	let sword_pivot_id = AnimationTargetId::from_iter(["Sword Pivot"]);

	let follow_through_time = 0.8;

	let mut left_attack_animation = AnimationClip::default();
	left_attack_animation.add_curve_to_target(
		sword_pivot_id,
		AnimatableCurve::new(
			animated_field!(Transform::rotation),
			EasingCurve::new(
				Quat::from_rotation_y(SwordSide::Left.angle()),
				Quat::from_rotation_y(SwordSide::Right.angle()),
				EaseFunction::QuarticOut,
			)
			.reparametrize_linear(Interval::new(0.0, follow_through_time).unwrap())
			.unwrap(),
		),
	);
	left_attack_animation.add_event(0.0, SwordStart);
	left_attack_animation.add_event(follow_through_time, SwordEnd);

	let mut right_attack_animation = AnimationClip::default();
	right_attack_animation.add_curve_to_target(
		sword_pivot_id,
		AnimatableCurve::new(
			animated_field!(Transform::rotation),
			EasingCurve::new(
				Quat::from_rotation_y(SwordSide::Right.angle()),
				Quat::from_rotation_y(SwordSide::Left.angle()),
				EaseFunction::QuarticOut,
			)
			.reparametrize_linear(Interval::new(0.0, follow_through_time).unwrap())
			.unwrap(),
		),
	);
	right_attack_animation.add_event(0.0, SwordStart);
	right_attack_animation.add_event(follow_through_time, SwordEnd);

	let mut graph = AnimationGraph::new();
	let left_attack_index = graph.add_clip(animations.add(left_attack_animation), 1.0, graph.root);
	let right_attack_index =
		graph.add_clip(animations.add(right_attack_animation), 1.0, graph.root);

	let sword_blade = commands
		.spawn((
			Name::new("Sword Blade"),
			Transform::from_translation(Vec3::NEG_Z * 1.)
				.with_rotation(Quat::from_rotation_x(PI / 2.)),
			Mesh3d(
				meshes.add(
					Capsule3d::new(0.1, 0.5)
						.mesh()
						.rings(1)
						.latitudes(8)
						.longitudes(16)
						.uv_profile(CapsuleUvProfile::Fixed),
				),
			),
			MeshMaterial3d(gridbox_material("red", materials, asset_server)),
			Sword::new(
				0.25,
				body,
				EntityHashSet::from_iter(vec![body]),
				left_attack_index,
				right_attack_index,
				asset_server.load("whoosh.mp3"),
			),
		))
		.id();

	let sword_pivot = commands
		.spawn((
			Name::new("Sword Pivot"),
			Transform::from_rotation(Quat::from_rotation_y(-PI * 0.5)),
			SwordPivot { blade: sword_blade },
			SweepPivot {
				sweeper_length: 0.2,
				sweep_depth: 0.5,
				sweep_height: 0.2,
			},
			AnimationGraphHandle(graphs.add(graph)),
			AnimationPlayer::default(),
			WeaponAnimation(left_attack_index),
		))
		.set_parent(body)
		.add_child(sword_blade)
		.observe(on_sword_start)
		.observe(on_sword_end)
		.id();
	commands.entity(sword_pivot).insert(AnimationTarget {
		id: sword_pivot_id,
		player: sword_pivot,
	});

	(sword_pivot, sword_blade)
}

#[derive(Event, Clone, Copy)]
struct SwordStart;

#[derive(Event, Clone, Copy)]
struct SwordEnd;

fn on_sword_start(
	trigger: Trigger<SwordStart>,
	sword_pivots: Query<&SwordPivot>,
	mut swords: Query<(&mut Sword, &GlobalTransform)>,
	fray: Query<&FrayMusic>,
	mut commands: Commands,
) {
	let sword_pivot_entity = trigger.entity();
	let sword_pivot = sword_pivots
		.get(sword_pivot_entity)
		.expect("Sword pivot not found");
	let sword_blade_entity = sword_pivot.blade;
	let (mut sword, transform) = swords.get_mut(sword_blade_entity).expect("Sword not found");

	let fray = fray.single();

	sword.current_slash_damage = fray.modify_fray_damage(sword.damage);
	sword.current_slash_modifier = fray.modify_fray_damage(1.0);

	commands.entity(sword_blade_entity).insert(DamageSweep::new(
		*transform,
		sword_pivot_entity,
		sword.allies.clone(),
		sword.wielder,
	));

	commands.spawn((
		Name::new("Sword Swing SFX"),
		AudioPlayer::new(sword.woosh_sound.clone()),
		PlaybackSettings::DESPAWN,
	));
}

fn on_sword_end(
	trigger: Trigger<SwordEnd>,
	mut sword_pivots: Query<(&SwordPivot, &mut WeaponAnimation)>,
	mut swords: Query<&mut Sword>,
	mut commands: Commands,
) {
	let sword_pivot_entity = trigger.entity();
	let (sword_pivot, mut animation) = sword_pivots
		.get_mut(sword_pivot_entity)
		.expect("Sword pivot not found");
	let sword_blade_entity = sword_pivot.blade;
	let mut sword = swords.get_mut(sword_blade_entity).expect("Sword not found");

	commands.entity(sword_blade_entity).insert(EndDamageSweep {
		damage: sword.current_slash_damage,
		fray_modifier: sword.current_slash_modifier,
	});

	sword.side = sword.side.other_side();
	animation.0 = match sword.side {
		SwordSide::Left => sword.left_swing_animation,
		SwordSide::Right => sword.right_swing_animation,
	};
}
