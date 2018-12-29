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

//! Structures for holding strings.
//!
//! This module contains [`string::Subsystem`], which manages [`StringId`]s. Since `StringId` is a
//! dumb POD, two wrapper are provided: [`Text`] and [`Name`]. While both can hold strings, `Name`
//! is optimized for strings with a numeric suffix. `Text`s implement [`Deref`]`<Target=`[`str`]`>`,
//! which is not the case for `Name`, because of the optimization.
//!
//! # Examples
//!
//! The string `Subsystem` can be created from a parent [`Logger`]:
//!
//! ```
//! # use astral_thirdparty::slog;
//! use astral::string;
//!
//!	# let logger = slog::Logger::root(slog::Discard, slog::o!());
//! # #[allow(unused_variables)]
//! let string_subsystem = string::Subsystem::new(64, &logger);
//! ```
//!
//! You can create a `StringId` with the `Subsystem`:
//! ```
//! # use astral_thirdparty::slog;
//!	# let logger = slog::Logger::root(slog::Discard, slog::o!());
//!	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
//! use astral::string::StringId;
//!
//! let id1 = StringId::new("foo", &string_subsystem);
//! let id2 = StringId::new("bar", &string_subsystem);
//! let id3 = StringId::new("foo", &string_subsystem);
//!
//! assert_ne!(id1, id2);
//! assert_eq!(id1, id3);
//! ```
//!
//!`Text` or `Name` can be created from a literal string:
//!
//! ```
//! # use astral_thirdparty::slog;
//!	# let logger = slog::Logger::root(slog::Discard, slog::o!());
//!	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
//! use astral::string::Text;
//!
//! let text = Text::new("foo", &string_subsystem);
//! assert_eq!(text, "foo");
//! ```
//!
//! A `Text` can be converted into [`&str`][`str`]:
//!
//! ```
//! # use astral_thirdparty::slog;
//!	# let logger = slog::Logger::root(slog::Discard, slog::o!());
//!	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
//! # use astral::string::Text;
//! let text = Text::new("foo", &string_subsystem);
//! let s: &str = text.as_str();
//!
//! assert_eq!("foo", s)
//! ```
//!
//! If you have a slice of valid UTF-8 bytes, you can make a `Text` or a `Name`
//! out of it.
//!
//! ```
//! # use astral_thirdparty::slog;
//!	# let logger = slog::Logger::root(slog::Discard, slog::o!());
//!	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
//! # use astral::string::Text;
//! let sparkle_heart = &[240, 159, 146, 150];
//!
//! // We know these bytes are valid, so we'll use `unwrap()`.
//! let sparkle_heart = Text::from_utf8(sparkle_heart, &string_subsystem).unwrap();
//!
//! assert_eq!("💖", sparkle_heart);
//!
//! let bytes = sparkle_heart.as_bytes();
//!
//! assert_eq!(bytes, [240, 159, 146, 150]);
//! ```
//!
//! [`Text`]: struct.Text.html
//! [`Name`]: struct.Name.html
//! [`StringId`]: struct.StringId.html
//! [`Logger`]: https://docs.rs/slog/2.4.1/slog/struct.Logger.html
//! [`string::Subsystem`]: struct.Subsystem.html
//! [`Deref`]: https://doc.rust-lang.org/nightly/std/ops/trait.Deref.html
//! [`str`]: https://doc.rust-lang.org/nightly/std/primitive.str.html
// TODO(#1): Use intra doc links

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

mod allocator;
mod entry;
mod entry_hash_table;
mod error;
mod name;
mod static_ref_vector;
mod string_id;
mod subsystem;
mod text;

#[doc]
pub use std::string::String;

pub use self::{
	entry::MAX_STRING_LENGTH,
	error::{Utf16Error, Utf8Error},
	name::Name,
	string_id::StringId,
	subsystem::Subsystem,
	text::Text,
};

use self::{
	allocator::Allocator,
	entry::{Entry, DATA_OFFSET},
	entry_hash_table::EntryHashTable,
	static_ref_vector::StaticRefVector,
};

const PAGE_SIZE: usize = 64 * 1024;
