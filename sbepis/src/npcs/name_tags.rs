use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use faker_rand::en_us::names::FirstName;
use meshtext::{Face, MeshGenerator, MeshText, TextSection};
use rand::seq::IteratorRandom;
use serde::Deserialize;

use crate::some_or_return;

#[derive(Resource)]
pub struct AvailableNamesAsset(Handle<AvailableNames>);

#[derive(Asset, Deserialize, TypePath)]
pub struct AvailableNames {
	names: Vec<String>,
}

#[derive(Resource)]
pub struct FontMeshGenerator(MeshGenerator<Face<'static>>);
impl FontMeshGenerator {
	pub fn new(font_data: &'static [u8]) -> Self {
		Self(MeshGenerator::new(font_data))
	}

	pub fn generate(&mut self, text: &str) -> (MeshText, Mesh) {
		let transform = Mat4::from_scale(Vec3::new(1.0, 1.0, 0.2)).to_cols_array();
		let mesh_text: MeshText = self
			.0
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
		Self::new(include_bytes!("../../assets/FiraSans-Regular.ttf")) // Cascadia Code is broken (Err: GlyphTriangulationError(PointOnFixedEdge(1)))
	}
}

#[derive(Component)]
pub struct SpawnNameTag;

pub fn load_names(mut commands: Commands, asset_server: Res<AssetServer>) {
	let asset: Handle<AvailableNames> = asset_server.load("supporters.names.ron");
	commands.insert_resource(AvailableNamesAsset(asset));
}

pub fn spawn_name_tags(
	mut commands: Commands,
	asset: Res<AvailableNamesAsset>,
	mut assets: ResMut<Assets<AvailableNames>>,
	entities: Query<Entity, With<SpawnNameTag>>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut font_mesh_generator: ResMut<FontMeshGenerator>,
) {
	let asset = some_or_return!(assets.get_mut(&asset.0));

	for entity in entities.iter() {
		commands.entity(entity).remove::<SpawnNameTag>();

		let name = {
			let opt = asset
				.names
				.iter()
				.enumerate()
				.choose(&mut rand::thread_rng())
				.map(|(i, name)| (i, name.clone()));
			if let Some((i, name)) = opt {
				asset.names.swap_remove(i);
				name
			} else {
				rand::random::<FirstName>().to_string()
			}
		};

		let (mesh_text, mesh) = font_mesh_generator.generate(&name);
		let scale = 0.2;

		commands
			.spawn(PbrBundle {
				mesh: meshes.add(mesh),
				material: materials.add(Color::srgb(1.0, 1.0, 1.0)),
				transform: Transform::from_xyz(mesh_text.bbox.size().x * scale * 0.5, 0.6, 0.0)
					.with_rotation(Quat::from_rotation_y(PI))
					.with_scale(Vec3::splat(scale)),
				..default()
			})
			.set_parent(entity);
	}
}
