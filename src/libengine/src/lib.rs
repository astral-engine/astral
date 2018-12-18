// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, December 2018

//! Facade for [`Engine`] and third party libraries.
//!
//! This is moved out of the [astral] crate to give other crates unified access `Engine` and
//! third-party libraries.

#![doc(
	html_no_source,
	html_logo_url = "https://astral-engine.github.io/docs/logo_astral.svg",
	html_favicon_url = "https://astral-engine.github.io/docs/logo.svg"
)]
#![warn(
	future_incompatible,
	nonstandard_style,
	rust_2018_compatibility,
	rust_2018_idioms,
	unused,
	box_pointers,
	macro_use_extern_crate,
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs,
	missing_doc_code_examples,
	single_use_lifetimes,
	trivial_casts,
	trivial_numeric_casts,
	unreachable_pub,
	unsafe_code,
	unused_import_braces,
	unused_lifetimes,
	unused_qualifications,
	unused_results,
	variant_size_differences,
	clippy::pedantic
)]

pub mod third_party;

mod engine;
pub use self::engine::Engine;
