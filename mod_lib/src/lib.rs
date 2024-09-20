#![allow(deprecated)]

use bevy::prelude::*;
use std::any::TypeId;

#[no_mangle]
pub fn _bevy_create_plugin() -> *mut dyn Plugin {
	// make sure the constructor is the correct type.
	let object = ModLib {};
	let boxed = Box::new(object);
	Box::into_raw(boxed)
}

pub struct ModLib;

impl Plugin for ModLib {
	fn build(&self, app: &mut App) {
		println!(
			"ModLib loaded! {:?} {:?}",
			app.world(),
			TypeId::of::<Schedules>()
		);
		app.add_systems(Update, setup);
		println!("ModLib system added!");
	}
}

fn setup(mut commands: Commands) {
	println!("Hello from mod_lib!");
	commands.spawn(Name::new("Sample modded entity"));
}
