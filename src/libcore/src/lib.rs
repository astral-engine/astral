// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

//! Low level support for mundane but crucial tasks.

#![cfg_attr(unstable, feature(align_offset))]
#![cfg_attr(unstable, feature(untagged_unions))]
#![doc(
	html_no_source,
	html_logo_url = "https://astral-engine.github.io/docs/logo_astral.svg",
	html_favicon_url = "https://astral-engine.github.io/docs/logo.svg",
	test(attr(deny(
		future_incompatible,
		nonstandard_style,
		rust_2018_compatibility,
		rust_2018_idioms,
		unused,
		macro_use_extern_crate,
		trivial_casts,
		trivial_numeric_casts,
		unused_import_braces,
		unused_lifetimes,
		unused_qualifications,
		variant_size_differences,
	)))
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
	missing_docs,
	missing_doc_code_examples,
	// single_use_lifetimes,
	trivial_casts,
	trivial_numeric_casts,
	unreachable_pub,
	// unsafe_code,
	unused_import_braces,
	unused_lifetimes,
	unused_qualifications,
	unused_results,
	variant_size_differences,
	clippy::pedantic
)]
#![allow(clippy::stutter)]

pub mod collections;
pub mod error;
pub mod hash;
pub mod math;
pub mod string;
mod system;

pub use self::system::System;
