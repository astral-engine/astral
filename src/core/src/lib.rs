// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

//! A collection of basic structures and functions.

#![cfg_attr(unstable, feature(align_offset))]
#![cfg_attr(unstable, feature(untagged_unions))]
#![doc(
	html_no_source,
	html_logo_url = "https://astral-engine.github.io/docs/logo_astral.svg",
	html_favicon_url = "https://astral-engine.github.io/docs/logo.svg",
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

extern crate either;
extern crate lazy_static;
extern crate num_traits;

pub mod collections;
pub mod error;
pub mod hash;
pub mod log;
pub mod math;
pub mod string;
