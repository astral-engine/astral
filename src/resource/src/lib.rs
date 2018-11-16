// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

#![cfg_attr(unstable, feature(non_exhaustive))]
#![cfg_attr(unstable, feature(doc_spotlight))]
#![doc(
	html_no_source,
	html_logo_url = "https://astral-engine.github.io/documentation/logo_astral.svg",
	html_favicon_url = "https://astral-engine.github.io/documentation/logo.svg",
	html_root_url = "https://astral-engine.github.io/documentation"
)]
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

extern crate serde;
extern crate walkdir;

extern crate astral_core;

pub mod assets;

mod declaration;
mod error;
mod load_data;
mod load_priority;
mod registry;
mod resource;
mod resource_id;

pub use self::{
	declaration::Declaration,
	error::{Error, ErrorKind, Result},
	load_data::LoadData,
	load_priority::LoadPriority,
	registry::{Loader, State},
	resource::Resource,
	resource_id::ResourceId,
};
