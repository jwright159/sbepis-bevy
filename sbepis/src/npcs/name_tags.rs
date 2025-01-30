use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy_butler::*;
use faker_rand::en_us::names::FirstName;
use meshtext::{Face, MeshGenerator, MeshText, TextSection};
use rand::seq::{IteratorRandom, SliceRandom};
use serde::Deserialize;

use crate::entity::spawner::EntitySpawnedSet;
use crate::entity::{EntityKilled, EntityKilledSet};
use crate::npcs::NpcPlugin;
use crate::some_or_return;

#[derive(Resource)]
pub struct NameTagAssets {
	names: Handle<AvailableNames>,

	generated_material: Handle<StandardMaterial>,
	past_material: Handle<StandardMaterial>,
	pgo_material: Handle<StandardMaterial>,
	captcha_material: Handle<StandardMaterial>,
	alchemiter_material: Handle<StandardMaterial>,
	denizen_materials: [Handle<StandardMaterial>; 4],
	master_material: Handle<StandardMaterial>,
}

#[derive(Asset, Deserialize, TypePath)]
pub struct AvailableNames {
	names: Vec<NameTag>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
enum NameTier {
	Past,
	Pgo,
	Captcha,
	Alchemiter,
	Denizen,
	Master,
}

#[derive(Component)]
pub struct SpawnNameTag;

#[derive(Component)]
pub struct NameTagged(pub NameTag);

#[derive(Debug, Clone, Deserialize)]
pub struct NameTag {
	name: String,
	tier: Option<NameTier>,
}

#[derive(Resource)]
#[resource(plugin = NpcPlugin)]
pub struct FontMeshGenerator {
	regular: MeshGenerator<Face<'static>>,
	bold: MeshGenerator<Face<'static>>,
}

impl FontMeshGenerator {
	pub fn new(regular_font_data: &'static [u8], bold_font_data: &'static [u8]) -> Self {
		Self {
			regular: MeshGenerator::new(regular_font_data),
			bold: MeshGenerator::new(bold_font_data),
		}
	}

	pub fn generate_regular(&mut self, text: &str) -> (MeshText, Mesh) {
		Self::generate(text, &mut self.regular)
	}

	pub fn generate_bold(&mut self, text: &str) -> (MeshText, Mesh) {
		Self::generate(text, &mut self.bold)
	}

	fn generate(text: &str, generator: &mut MeshGenerator<Face<'static>>) -> (MeshText, Mesh) {
		let transform = Mat4::from_scale(Vec3::new(1.0, 1.0, 0.2)).to_cols_array();
		let mesh_text: MeshText = generator
			.generate_section(text, false, Some(&transform))
			.unwrap();

		let vertices = mesh_text.vertices.clone();
		let positions: Vec<[f32; 3]> = vertices.chunks(3).map(|c| [c[0], c[1], c[2]]).collect();
		let uvs = vec![[0.0, 0.0]; positions.len()];

		let mut mesh = Mesh::new(
			bevy::render::render_resource::PrimitiveTopology::TriangleList,
			RenderAssetUsages::RENDER_WORLD,
		);
		mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
		mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
		mesh.compute_flat_normals();

		(mesh_text, mesh)
	}
}
impl Default for FontMeshGenerator {
	fn default() -> Self {
		// Cascadia Code is broken (Err: GlyphTriangulationError(PointOnFixedEdge(1)))
		Self::new(
			include_bytes!("../../assets/FiraSans-Regular.ttf"),
			include_bytes!("../../assets/FiraSans-Bold.ttf"),
		)
	}
}

#[system(
	plugin = NpcPlugin, schedule = Startup,
)]
fn load_names(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	let names: Handle<AvailableNames> = asset_server.load("supporters.names.ron");
	commands.insert_resource(NameTagAssets {
		names,
		generated_material: materials.add(Color::srgb(0.4, 0.4, 0.4)),
		past_material: materials.add(Color::WHITE),
		pgo_material: materials.add(Color::from(Srgba::hex("4bec13").unwrap())),
		captcha_material: materials.add(Color::from(Srgba::hex("ff067c").unwrap())),
		alchemiter_material: materials.add(Color::from(Srgba::hex("03a9f4").unwrap())),
		denizen_materials: [
			materials.add(Color::from(Srgba::hex("0715cd").unwrap())),
			materials.add(Color::from(Srgba::hex("b536da").unwrap())),
			materials.add(Color::from(Srgba::hex("e00707").unwrap())),
			materials.add(Color::from(Srgba::hex("4ac925").unwrap())),
		],
		master_material: materials.add(Color::from(Srgba::hex("ff0000").unwrap())),
	});
}

#[system(
	plugin = NpcPlugin, schedule = Update,
	after = EntitySpawnedSet,
)]
fn spawn_name_tags(
	mut commands: Commands,
	asset: Res<NameTagAssets>,
	mut assets: ResMut<Assets<AvailableNames>>,
	entities: Query<Entity, With<SpawnNameTag>>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut font_mesh_generator: ResMut<FontMeshGenerator>,
) {
	let names = some_or_return!(assets.get_mut(&asset.names));

	for entity in entities.iter() {
		let name_tag = {
			let opt = names
				.names
				.iter()
				.enumerate()
				.choose(&mut rand::thread_rng())
				.map(|(i, name)| (i, name.clone()));
			if let Some((i, name_tag)) = opt {
				names.names.swap_remove(i);
				name_tag
			} else {
				NameTag {
					name: rand::random::<FirstName>().to_string(),
					tier: None,
				}
			}
		};

		let (mesh_text, mesh) = match name_tag.tier {
			None => font_mesh_generator.generate_regular(&name_tag.name),
			Some(NameTier::Past) => font_mesh_generator.generate_regular(&name_tag.name),
			Some(NameTier::Pgo) => font_mesh_generator.generate_regular(&name_tag.name),
			Some(NameTier::Captcha) => font_mesh_generator.generate_regular(&name_tag.name),
			Some(NameTier::Alchemiter) => font_mesh_generator.generate_bold(&name_tag.name),
			Some(NameTier::Denizen) => font_mesh_generator.generate_regular(&name_tag.name),
			Some(NameTier::Master) => font_mesh_generator.generate_bold(&name_tag.name),
		};
		let material = match name_tag.tier {
			None => asset.generated_material.clone(),
			Some(NameTier::Past) => asset.past_material.clone(),
			Some(NameTier::Pgo) => asset.pgo_material.clone(),
			Some(NameTier::Captcha) => asset.captcha_material.clone(),
			Some(NameTier::Alchemiter) => asset.alchemiter_material.clone(),
			Some(NameTier::Denizen) => asset
				.denizen_materials
				.choose(&mut rand::thread_rng())
				.unwrap()
				.clone(),
			Some(NameTier::Master) => asset.master_material.clone(),
		};
		let scale = match name_tag.tier {
			None => 0.2,
			Some(NameTier::Past) => 0.2,
			Some(NameTier::Pgo) => 0.2,
			Some(NameTier::Captcha) => 0.2,
			Some(NameTier::Alchemiter) => 0.2,
			Some(NameTier::Denizen) => 0.3,
			Some(NameTier::Master) => 0.3,
		};

		commands
			.spawn((
				Mesh3d(meshes.add(mesh)),
				MeshMaterial3d(material),
				Transform::from_xyz(mesh_text.bbox.size().x * scale * 0.5, 1.1, 0.0)
					.with_rotation(Quat::from_rotation_y(PI))
					.with_scale(Vec3::splat(scale)),
			))
			.set_parent(entity);

		commands
			.entity(entity)
			.remove::<SpawnNameTag>()
			.insert(NameTagged(name_tag));
	}
}

#[system(
	plugin = NpcPlugin, schedule = Update,
	after = EntityKilledSet,
)]
fn add_killed_name_back(
	mut ev_killed: EventReader<EntityKilled>,
	mut names: ResMut<Assets<AvailableNames>>,
	assets: Res<NameTagAssets>,
	name_tagged: Query<&NameTagged>,
) {
	let names = names.get_mut(&assets.names).unwrap();
	for ev in ev_killed.read() {
		if let Ok(name_tagged) = name_tagged.get(ev.0) {
			names.names.push(name_tagged.0.clone());
		}
	}
}
