use bevy::pbr::MaterialPlugin;
use bevy_butler::*;
use bevy_common_assets::ron::RonAssetPlugin;
use name_tags::{AvailableNames, CandyMaterial};

pub mod consort;
pub mod imp;
pub mod name_tags;

#[butler_plugin(build(
	add_plugins(RonAssetPlugin::<AvailableNames>::new(&["names.ron"])),
	add_plugins(MaterialPlugin::<CandyMaterial>::default()),
))]
pub struct NpcPlugin;
