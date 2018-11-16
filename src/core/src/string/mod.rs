// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

//! Structures for holding strings.
//!
//! This module contains the [`Name`] and the [`Text`] type. While both can hold
//! strings, `Name` is optimized for strings with a numeric suffix. `Text`s
//! implement [`Deref`]`<Target=`[`str`]`>`, which is not the case for `Name`,
//! because of the optimization.
//!
//! # Examples
//!
//! There are multiple ways to create a new `Text` or a new `Name` from
//! a string literal:
//!
//! ```
//! # extern crate astral;
//! use astral::core::string::{Text, Name};
//!
//! let t = Text::from("foo");
//! let n: Name = "foo".into();
//! assert_eq!(t, n);
//! ```
//!
//! A `Text` can be converted into [`&'static str`][`str`]:
//!
//! ```
//! # extern crate astral;
//! use astral::core::string::Text;
//!
//! let n = Text::from("foo");
//! let s: &'static str = n.as_str();
//!
//! assert_eq!("foo", s)
//! ```
//!
//! If you have a slice of valid UTF-8 bytes, you can make a `Text` or a `Name` out of it.
//!
//! ```
//! # extern crate astral;
//! use astral::core::string::Text;
//!
//! let sparkle_heart = &[240, 159, 146, 150];
//!
//! // We know these bytes are valid, so we'll use `unwrap()`.
//! let sparkle_heart = Text::from_utf8(sparkle_heart).unwrap();
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
//! [`Deref`]: https://doc.rust-lang.org/nightly/std/ops/trait.Deref.html
//! [`str`]: https://doc.rust-lang.org/nightly/std/primitive.str.html

mod allocator;
mod entry;
mod entry_hash_table;
mod error;
mod name;
mod static_ref_vector;
mod text;

use std::ptr;

use lazy_static::lazy_static;

pub use self::{
	entry::MAX_STRING_LENGTH,
	error::{Utf16Error, Utf8Error},
	name::Name,
	text::Text,
};

use self::{
	allocator::Allocator,
	entry::{Entry, DATA_OFFSET},
	entry_hash_table::EntryHashTable,
	static_ref_vector::StaticRefVector,
};

/// The maximum number of unique strings like [`Text`] or [`Name`].
///
/// [`Text`]: struct.Text.html
/// [`Name`]: struct.Name.html
pub const MAX_STRINGS: usize = 1024 * 1024;

const PAGE_SIZE: usize = 64 * 1024;

// TODO(#8): Use `Allocator::new()`
static mut ALLOCATOR: Allocator = Allocator {
	current_pool_start: ptr::null_mut(),
	current_pool_end: ptr::null_mut(),
};

lazy_static! {
	static ref ENTRY_REFERENCE_MAP: StaticRefVector<'static, Entry> =
		StaticRefVector::new(MAX_STRINGS);
	static ref ENTRY_HASH_TABLE: EntryHashTable = EntryHashTable::new();
}
