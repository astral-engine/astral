// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::{
	hint, mem,
	num::NonZeroU32,
	slice, str,
	sync::atomic::{self, AtomicPtr},
};

use super::{ALLOCATOR, PAGE_SIZE};

pub(super) const DATA_OFFSET: usize = 6 + mem::size_of::<AtomicPtr<Entry>>();
/// The maximum length of one string like  [`Text`] or [`Name`].
///
/// [`Text`]: string::Text
/// [`Name`]: string::Name.
pub const MAX_STRING_LENGTH: usize = PAGE_SIZE - DATA_OFFSET;

/// An entry for a `Name`.
///
/// It stores the index into the global entry table, the length of the underlying
/// string and the string data.
// Don't forget to adjust `MAX_STRING_LENGTH` when adding fields to match `PAGE_SIZE` (64KB)
#[repr(C)]
pub(super) struct Entry {
	pub(super) next: AtomicPtr<Entry>,
	pub(super) index: Option<NonZeroU32>,
	pub(super) len: u16,

	pub(super) data: [u8; MAX_STRING_LENGTH],
	// CAUTION: No fields must be added after `data`. `Entry` is only allocated according to the
	// string length. All fields that are stored in memory after `data` will end with a
	// segmentation error at best.
}

impl Entry {
	/// Allocates a new `Entry` and returns a pointer to it. Allocating needs external
	/// synchronization.
	///
	/// # Safety
	///
	/// This is unsafe because allocating is not thread safe.
	pub(super) unsafe fn allocate(string: &str) -> *mut Self {
		if string.len() > MAX_STRING_LENGTH {
			log::warn!(
				"name is greater than the allowed size of {}: {:?}",
				MAX_STRING_LENGTH,
				string
			);
			ALLOCATOR.allocate(&string[0..MAX_STRING_LENGTH])
		} else {
			ALLOCATOR.allocate(string)
		}
	}

	/// Returns the index of the entry.
	pub(super) fn index(&self) -> NonZeroU32 {
		self.index.unwrap_or_else(|| {
			debug_assert!(false, "Entry was not initialized");
			unsafe { hint::unreachable_unchecked() }
		})
	}

	/// Returns a pointer to the next hash entry in the global hash bucket.
	pub(super) fn next(&self) -> &AtomicPtr<Self> {
		&self.next
	}

	/// Returns `true` if it has a length of zero.
	///
	/// Returns `false` otherwise.
	pub(super) fn len(&self) -> u16 {
		self.len
	}

	/// Returns the length in bytes.
	pub(super) fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Returns a string from its length and data.
	pub(super) fn as_str(&self) -> &str {
		unsafe {
			let slice = slice::from_raw_parts(self.data.as_ptr(), self.len as usize);
			str::from_utf8_unchecked(slice)
		}
	}

	/// Returns an Iterator over all known entries with the same hash value.
	pub(super) fn iter(&'static self) -> impl Iterator<Item = &'static Self> {
		Entries { current: self }
	}
}

struct Entries {
	current: *const Entry,
}

impl Iterator for Entries {
	type Item = &'static Entry;

	fn next(&mut self) -> Option<Self::Item> {
		if self.current.is_null() {
			None
		} else {
			let current = unsafe { &*self.current };
			self.current = current.next().load(atomic::Ordering::Relaxed);
			Some(current)
		}
	}
}
