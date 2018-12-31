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
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

//! Utility traits, structures and functions used in the Astral Engine.

#![doc(
	html_no_source,
	html_logo_url = "https://astral-engine.github.io/docs/logo_astral.svg",
	html_favicon_url = "https://astral-engine.github.io/docs/logo.svg",
	test(attr(
		deny(
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
		),
		allow(unused_extern_crates)
	))
)]
#![warn(
	future_incompatible,
	nonstandard_style,
	rust_2018_compatibility,
	rust_2018_idioms,
	unused,
	macro_use_extern_crate,
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs,
	// missing_doc_code_examples,
	// single_use_lifetimes,
	trivial_casts,
	trivial_numeric_casts,
	unreachable_pub,
	unused_import_braces,
	unused_lifetimes,
	unused_qualifications,
	unused_results,
	variant_size_differences,
	clippy::pedantic
)]

pub mod hash;
