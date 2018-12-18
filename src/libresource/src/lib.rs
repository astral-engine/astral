// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

//! Resource creating and loading.
//!
//! See the [resource system] for further details.
//!
//! [resource system]: struct.System.html

#![cfg_attr(unstable, feature(non_exhaustive))]
#![cfg_attr(unstable, feature(doc_spotlight))]
#![doc(
	html_no_source,
	html_logo_url = "https://astral-engine.github.io/docs/logo_astral.svg",
	html_favicon_url = "https://astral-engine.github.io/docs/logo.svg",
	html_root_url = "https://astral-engine.github.io/docs"
)]
#![warn(
	future_incompatible,
	nonstandard_style,
	rust_2018_compatibility,
	rust_2018_idioms,
	unused,
	// box_pointers,
	macro_use_extern_crate,
	missing_copy_implementations,
	missing_debug_implementations,
	// missing_docs,
	// missing_doc_code_examples,
	// single_use_lifetimes,
	trivial_casts,
	trivial_numeric_casts,
	// unreachable_pub,
	// unsafe_code,
	unused_import_braces,
	unused_lifetimes,
	unused_qualifications,
	unused_results,
	variant_size_differences,
	clippy::pedantic
)]

pub mod assets;

mod declaration;
mod error;
mod load_data;
mod load_priority;
mod registry;
mod resource;
mod resource_id;
mod system;

pub use self::{
	declaration::Declaration,
	error::{Error, ErrorKind, Result},
	load_data::LoadData,
	load_priority::LoadPriority,
	registry::{Loader, State},
	resource::Resource,
	resource_id::ResourceId,
	system::System,
};
