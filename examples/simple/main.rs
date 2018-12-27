// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

#![warn(
	bad_style,
	nonstandard_style,
	warnings,
	rust_2018_compatibility,
	rust_2018_idioms,
	single_use_lifetimes,
	trivial_casts,
	trivial_numeric_casts,
	variant_size_differences,
	absolute_paths_not_starting_with_crate,
	future_incompatible,
	unused,
	clippy::pedantic
)]
#![allow(clippy::cast_precision_loss)]

use std::error::Error;

use astral::{
	core::string::Name,
	resource::assets::{FileSystem, VirtualFileSystem},
};

fn app(engine: &astral::Engine) -> Result<(), Box<dyn Error>> {
	let core_system = astral::core::System::new(engine);
	let string_subsystem = astral::core::string::Subsystem::new(10 * 1024 * 1024, &core_system);

	let resource_system = astral::resource::System::new(&engine);
	let asset_subsystem = astral::resource::assets::Subsystem::new(&resource_system);

	let directory = Name::new("assets", &string_subsystem);
	let files = FileSystem::new(directory, &asset_subsystem, &string_subsystem)?
		.iter()?
		.collect::<Vec<_>>();

	let mut counter = 0_u32;
	for file in &files {
		counter += 1;
		info!(engine.logger(), "file"; "name" => ?file, "count" => counter);
	}

	// catalog[core_namespace].add_virtual_file_system(filesystem)?;

	// let mut registry = Loader::<TextFile, Option<&str>>::new(
	// 	|string| Ok(TextFile::from(string.unwrap().to_string())),
	// 	|_, read| {
	// 		let mut string = String::new();
	// 		read.read_to_string(&mut string)?;
	// 		Ok(TextFile::from(string))
	// 	},
	// );
	// registry.set_catalog(catalog);

	// let cube_model = Location::from_string(core_namespace, "models/cube.obj");
	// let _cube_model_resource_id = registry.declare_asset(cube_model);
	// let _constant_resource_id = registry.declare_resource(Name::from("constant1"));

	// info!("Hello World");

	// dbg!(std::mem::size_of_val(&cube_model));
	// dbg!(std::mem::size_of_val(&registry));
	// dbg!(std::mem::size_of::<Location>());
	// dbg!(std::mem::size_of::<ResourceId>());
	// dbg!(std::mem::size_of::<astral::resource::assets::Error>());

	Ok(())
}

use astral::third_party::slog::{error, info, o, trace, warn, Drain, Logger};

fn main() {
	let decorator = slog_term::TermDecorator::new().build();
	let drain = slog_term::CompactFormat::new(decorator).build().fuse();
	let drain = slog_async::Async::new(drain)
		.chan_size(64 * 1024)
		.overflow_strategy(slog_async::OverflowStrategy::Block)
		.build()
		.fuse();

	let log = Logger::root(drain, o!());
	trace!(log, "test");

	let engine = astral::Engine::new(&log);

	if let Err(err) = app(&engine) {
		let mut err: &dyn std::error::Error = err.as_ref();
		error!(log, "{}", err);
		while let Some(source) = err.source() {
			error!(log, "  {}", source);
			err = source;
		}
		std::process::exit(1);
	}
}
