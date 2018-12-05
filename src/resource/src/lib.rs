// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

#![cfg_attr(unstable, feature(non_exhaustive))]
#![cfg_attr(unstable, feature(doc_spotlight))]
#![doc(
	html_no_source,
	html_logo_url = "https://astral-engine.github.io/docs/logo_astral.svg",
	html_favicon_url = "https://astral-engine.github.io/docs/logo.svg",
	html_root_url = "https://astral-engine.github.io/docs"
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
	unused,
	clippy::pedantic
)]
#![allow(single_use_lifetimes)]

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
