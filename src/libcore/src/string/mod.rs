// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
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
//! The string `Subsystem` can be created from [`core::System`]:
//!
//! ```
//! # use astral::third_party::slog;
//!	# let logger = slog::Logger::root(slog::Discard, slog::o!());
//! use astral::{
//! 	Engine,
//! 	core::{self, string},
//! };
//!
//! let engine = Engine::new(&logger);
//! let core_system = core::System::new(&engine);
//! # #[allow(unused_variables)]
//! let string_subsystem = string::Subsystem::new(64, &core_system);
//! ```
//!
//! You can create a `StringId` with the `Subsystem`:
//! ```
//! # use astral::{third_party::slog, Engine, core::{System, string}};
//!	# let logger = slog::Logger::root(slog::Discard, slog::o!());
//!	# let engine = Engine::new(&logger);
//!	# let system = System::new(&engine);
//!	# let string_subsystem = string::Subsystem::new(64, &system);
//! use astral::core::string::StringId;
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
//! # use astral::{third_party::slog, Engine, core::{System, string}};
//!	# let logger = slog::Logger::root(slog::Discard, slog::o!());
//!	# let engine = Engine::new(&logger);
//!	# let system = System::new(&engine);
//!	# let string_subsystem = string::Subsystem::new(64, &system);
//! use astral::core::string::Text;
//!
//! let text = Text::new("foo", &string_subsystem);
//! assert_eq!(text, "foo");
//! ```
//!
//! A `Text` can be converted into [`&str`][`str`]:
//!
//! ```
//! # use astral::{third_party::slog, Engine, core::{System, string}};
//!	# let logger = slog::Logger::root(slog::Discard, slog::o!());
//!	# let engine = Engine::new(&logger);
//!	# let system = System::new(&engine);
//!	# let string_subsystem = string::Subsystem::new(64, &system);
//! use astral::core::string::Text;
//!
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
//! # use astral::{third_party::slog, Engine, core::{System, string}};
//!	# let logger = slog::Logger::root(slog::Discard, slog::o!());
//!	# let engine = Engine::new(&logger);
//!	# let system = System::new(&engine);
//!	# let string_subsystem = string::Subsystem::new(64, &system);
//! use astral::core::string::Text;
//!
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
//! [`core::System`]: ../struct.System.html
//! [`string::Subsystem`]: struct.Subsystem.html
//! [`Deref`]: https://doc.rust-lang.org/nightly/std/ops/trait.Deref.html
//! [`str`]: https://doc.rust-lang.org/nightly/std/primitive.str.html
// TODO(#1): Use intra doc links

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
