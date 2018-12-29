// Copyright (c) Astral Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// // Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

// #![warn(
// 	bad_style,
// 	nonstandard_style,
// 	warnings,
// 	rust_2018_compatibility,
// 	rust_2018_idioms,
// 	single_use_lifetimes,
// 	trivial_casts,
// 	trivial_numeric_casts,
// 	variant_size_differences,
// 	absolute_paths_not_starting_with_crate,
// 	future_incompatible,
// 	unused,
// 	clippy::pedantic
// )]
// #![allow(clippy::cast_precision_loss)]

// use std::error::Error;

// use astral::{
// 	core::string::Name,
// 	resource::assets::{FileSystem, VirtualFileSystem},
// 	third_party::{
// 		rayon,
// 		slog::{error, info, o, trace, warn, Drain, Logger},
// 	},
// };

// fn app(engine: &astral::Engine) -> Result<(), Box<dyn Error>> {
// 	let core_system = astral::core::System::new(engine);
// 	let string_subsystem = astral::core::string::Subsystem::new(10 * 1024 * 1024, &core_system);

// 	let resource_system = astral::resource::System::new(&engine);
// 	let asset_subsystem = astral::resource::assets::Subsystem::new(&resource_system);

// 	let directory = Name::new("assets", &string_subsystem);
// 	let files = FileSystem::new(directory, &asset_subsystem, &string_subsystem)?
// 		.iter()?
// 		.collect::<Vec<_>>();

// 	let mut counter = 0_u32;
// 	for file in &files {
// 		counter += 1;
// 		info!(engine.logger(), "file"; "name" => ?file, "count" => counter);
// 	}

// 	let pool = rayon::ThreadPoolBuilder::new()
// 		.num_threads(2)
// 		.build()
// 		.unwrap();

// 	let ok: Vec<i32> = vec![1, 2, 3];
// 	pool.scope(|s| {
// 		let bad: Vec<i32> = vec![4, 5, 6];
// 		s.spawn(|_| {
// 			let bad = bad;
// 			info!(engine.logger(), "ok: {:?}", ok);
// 			info!(engine.logger(), "bad: {:?}", bad);
// 		});
// 		info!(engine.logger(), "borrowed {:?}", ok);
// 	});

// 	// catalog[core_namespace].add_virtual_file_system(filesystem)?;

// 	// let mut registry = Loader::<TextFile, Option<&str>>::new(
// 	// 	|string| Ok(TextFile::from(string.unwrap().to_string())),
// 	// 	|_, read| {
// 	// 		let mut string = String::new();
// 	// 		read.read_to_string(&mut string)?;
// 	// 		Ok(TextFile::from(string))
// 	// 	},
// 	// );
// 	// registry.set_catalog(catalog);

// 	// let cube_model = Location::from_string(core_namespace, "models/cube.obj");
// 	// let _cube_model_resource_id = registry.declare_asset(cube_model);
// 	// let _constant_resource_id = registry.declare_resource(Name::from("constant1"));

// 	// info!("Hello World");

// 	// dbg!(std::mem::size_of_val(&cube_model));
// 	// dbg!(std::mem::size_of_val(&registry));
// 	// dbg!(std::mem::size_of::<Location>());
// 	// dbg!(std::mem::size_of::<ResourceId>());
// 	// dbg!(std::mem::size_of::<astral::resource::assets::Error>());

// 	Ok(())
// }

// fn main() {
// 	let decorator = slog_term::TermDecorator::new().build();
// 	let drain = slog_term::CompactFormat::new(decorator).build().fuse();
// 	// let drain = slog_async::Async::new(drain)
// 	// 	.chan_size(64 * 1024)
// 	// 	.overflow_strategy(slog_async::OverflowStrategy::Block)
// 	// 	.build()
// 	// 	.fuse();
// 	let drain = std::sync::Mutex::new(drain).fuse();

// 	let log = Logger::root(drain, o!());
// 	trace!(log, "test");

// 	let engine = astral::Engine::new(&log);

// 	if let Err(err) = app(&engine) {
// 		let mut err: &dyn std::error::Error = err.as_ref();
// 		error!(log, "{}", err);
// 		while let Some(source) = err.source() {
// 			error!(log, "  {}", source);
// 			err = source;
// 		}
// 		std::process::exit(1);
// 	}
// }

fn main() {}
