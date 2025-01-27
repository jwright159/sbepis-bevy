use bevy_butler::*;
use bevy_common_assets::ron::RonAssetPlugin;
use name_tags::AvailableNames;

pub mod consort;
pub mod imp;
pub mod name_tags;

#[butler_plugin(build(
	add_plugins(RonAssetPlugin::<AvailableNames>::new(&["names.ron"])),
))]
pub struct NpcPlugin;
