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
	resource::{
		assets::{Catalog, FileSystem, Location, Namespace},
		Loader,
		Resource,
		ResourceId,
	},
};

macro_rules! dbg {
	($val:expr) => {
		// Use of `match` here is intentional because it affects the lifetimes
		// of temporaries - https://stackoverflow.com/a/48732525/1063961
		match $val {
			tmp => {
				eprintln!(
					"[{}:{}] {} = {:#?}",
					file!(),
					line!(),
					stringify!($val),
					&tmp
				);
				tmp
				}
			}
	};
}

#[derive(Debug)]
struct TextFile {
	string: String,
}

impl Resource for TextFile {
	type LoadData = String;
}

impl From<String> for TextFile {
	fn from(string: String) -> Self {
		Self { string }
	}
}

fn app(engine: &astral::Engine) -> Result<(), Box<dyn Error>> {
	let core_system = astral::core::System::new(engine);
	let string_subsystem = astral::core::string::Subsystem::new(&core_system);

	let resource_system = astral::resource::System::new(&engine);
	let asset_subsystem = astral::resource::assets::Subsystem::new(&resource_system);

	let video_system = astral::video::System::new(&engine);
	// info!(
	// 	"strings allocated: {}, memory used: {}",
	// 	astral::core::string::allocated_strings(),
	// 	astral::core::string::used_memory(),
	// );
	// info!("size of usize: {}", std::mem::size_of::<usize>());
	// dbg!(Name::default());
	// info!(
	// 	"strings allocated: {}, memory used: {}",
	// 	astral::core::string::allocated_strings(),
	// 	astral::core::string::used_memory(),
	// );
	// dbg!(Name::from("string1"));
	// dbg!(Name::from("1234"));
	// dbg!(Name::from("0"));
	// dbg!(Name::from("1"));
	// dbg!(Name::from("01"));
	// dbg!(Name::from("string2300040000000000000000"));
	// dbg!(Name::from("string40"));
	// dbg!(Name::from("string60"));
	// dbg!(Name::from("string80"));
	// dbg!(Name::from("string0234"));
	// info!(
	// 	"strings allocated: {}, memory used: {} KB in {} chunks",
	// 	astral::core::string::allocated_strings(),
	// 	astral::core::string::used_memory() as f64 / 1024.0,
	// 	astral::core::string::used_memory_chunks()
	// );

	// return Ok(());
	// unsafe {
	// 	dbg!(astral::core::string::Entry::allocate(
	// 		"testtesttesttesttesttesttesttesttesttesttesttesttesttestte",
	// 		0
	// 	));
	// 	dbg!(astral::core::string::Entry::allocate(
	// 		"testtesttesttesttesttesttesttesttesttesttesttesttesttesttes",
	// 		0
	// 	));
	// 	dbg!(astral::core::string::Entry::allocate("test", 0));
	// }

	let mut catalog = Catalog::new();
	let core_namespace = catalog.add_namespace(Namespace::new());

	let filesystem = FileSystem::new(&asset_subsystem, "assets", true)?;
	catalog[core_namespace].add_virtual_file_system(filesystem)?;

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

use slog::{crit as critical, debug, error, info, trace, warn, Drain};

fn main() {
	let decorator = slog_term::TermDecorator::new().build();
	let drain = slog_term::CompactFormat::new(decorator).build().fuse();
	// let drain = slog_async::Async::new(drain).build().fuse();

	let log = slog::Logger::root(std::sync::Mutex::new(drain).fuse(), slog::o!());

	let engine = astral::Engine::new(&log);

	// let decorator = slog_term::PlainDecorator::new(std::io::stdout());
	// let drain = slog_term::CompactFormat::new(decorator).build().fuse();
	// let drain = slog_async::Async::new(drain).build().fuse();

	// let log = slog::Logger::root(drain, slog::o!("version" => "0.0.0"));

	// slog::info!(log, "test");
	// let logger = Box::new(log::TerminalLogger::default());
	// log::set_max_level(LevelFilter::Trace);
	// info!("test");
	// if let Err(err) = log::set_logger(&logger) {
	// eprintln!("Could not initialize logging: {}", err);
	// }
	// let rustc = rustc_version::version().unwrap();
	// info!("rustc: {}.{}.{}", rustc.major, rustc.minor, rustc.patch);
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
