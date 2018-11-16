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
	unused
)]
// TODO(#7): Use tool-lints
#![cfg_attr(feature = "cargo-clippy", warn(pedantic))]
#![cfg_attr(feature = "cargo-clippy", allow(stutter))]
#![allow(unused_extern_crates, single_use_lifetimes)]

extern crate astral;

use std::{error::Error, str};

use astral::core::log::{self, error, info, LevelFilter};

use astral::{
	core::string::Name,
	resource::{
		assets::{Catalog, FileSystem, Location, Namespace},
		Loader, Resource, ResourceId,
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

fn app() -> Result<(), Box<dyn Error>> {
	info!("size of usize: {}", std::mem::size_of::<usize>());
	dbg!(Name::default());
	dbg!(Name::from("string1"));
	dbg!(Name::from("1234"));
	dbg!(Name::from("0"));
	dbg!(Name::from("1"));
	dbg!(Name::from("01"));
	dbg!(Name::from("string23000400000000000000000"));
	dbg!(Name::from("string40"));
	dbg!(Name::from("string60"));
	dbg!(Name::from("string80"));
	dbg!(Name::from("string0234"));

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

	catalog[core_namespace]
		.add_virtual_file_system(FileSystem::new("assets", true)?)?;

	let mut registry = Loader::<TextFile, Option<&str>>::new(
		|string| Ok(TextFile::from(string.unwrap().to_string())),
		|_, read| {
			let mut string = String::new();
			read.read_to_string(&mut string)?;
			Ok(TextFile::from(string))
		},
	);
	registry.set_catalog(catalog);

	let cube_model = Location::from_string(core_namespace, "models/cube.obj");
	let _cube_model_resource_id = registry.declare_asset(cube_model);
	let _constant_resource_id =
		registry.declare_resource(Name::from("constant1"));

	info!("Hello World");

	dbg!(std::mem::size_of_val(&cube_model));
	dbg!(std::mem::size_of_val(&registry));
	dbg!(std::mem::size_of::<Location>());
	dbg!(std::mem::size_of::<ResourceId>());
	dbg!(std::mem::size_of::<astral::resource::assets::Error>());

	Ok(())
}

fn main() {
	let logger = Box::new(log::TerminalLogger::default());
	log::set_max_level(LevelFilter::Trace);
	if let Err(err) = log::set_boxed_logger(logger) {
		eprintln!("Could not initialize logging: {}", err);
	}
	if let Err(err) = app() {
		let mut err: &dyn std::error::Error = err.as_ref();
		error!("{}", err);
		while let Some(source) = err.source() {
			error!("  {}", source);
			err = source;
		}
		std::process::exit(1);
	}
}
