// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

#![doc(
	html_no_source,
	html_logo_url = "https://astral-engine.github.io/docs/logo_astral.svg",
	html_favicon_url = "https://astral-engine.github.io/docs/logo.svg"
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

pub mod core {
	#![allow(clippy::stutter)]

	//! Low-level support systems for manage mundane but crucial tasks.
	#[doc(inline)]
	pub use astral_core::*;
}

pub mod resource {
	//! Support systems for creating and loading different resources.
	#[doc(inline)]
	pub use astral_resource::*;
}
