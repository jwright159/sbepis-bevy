#![allow(deprecated)]

use std::any::TypeId;
use std::ffi::OsStr;

use bevy::prelude::*;
use libloading::{Library, Symbol};

fn main() {
	let mut app = App::new();
	app.add_plugins((
		DefaultPlugins,
		bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
	));
	unsafe {
		// app.load_plugin("./target/debug/libmod_lib.so");

		match dynamically_load_plugin("./target/debug/libmod_lib.so") {
			Ok((lib, plugin)) => {
				info!(
					"Plugin loaded! {:?} {:?}",
					app.world(),
					TypeId::of::<Schedules>()
				);
				std::mem::forget(lib);
				plugin.build(&mut app);
			}
			Err(e) => error!("Failed to load plugin: {:?}", e),
		}
	}
	bevy_mod_debugdump::print_schedule_graph(&mut app, Update);
	app.run();
}

pub unsafe fn dynamically_load_plugin<P: AsRef<OsStr>>(
	path: P,
) -> Result<(Library, Box<dyn Plugin>), libloading::Error> {
	// SAFETY: Caller must follow the safety requirements of Library::new.
	let lib = unsafe { Library::new(path)? };

	// SAFETY: Loaded plugins are not allowed to specify `_bevy_create_plugin` symbol manually, but
	// must instead automatically generate it through `DynamicPlugin`.
	let func: Symbol<unsafe fn() -> *mut dyn Plugin> = unsafe { lib.get(b"_bevy_create_plugin")? };

	// SAFETY: `func` is automatically generated and is guaranteed to return a pointer created using
	// `Box::into_raw`.
	let plugin = unsafe { Box::from_raw(func()) };

	Ok((lib, plugin))
}
