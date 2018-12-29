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

use std::{
	hint,
	mem,
	slice,
	str,
	sync::atomic::{self, AtomicPtr},
};

use super::{StringId, PAGE_SIZE};

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
// CAUTION: Don't forget to adjust `MAX_STRING_LENGTH` when adding fields to match `PAGE_SIZE` (64KB)
#[repr(C)]
pub(super) struct Entry {
	pub(super) next: AtomicPtr<Entry>,
	pub(super) id: Option<StringId>,
	pub(super) len: u16,

	pub(super) data: [u8; MAX_STRING_LENGTH],
	// CAUTION: No fields must be added after `data`. `Entry` is only allocated according to the
	// string length. All fields that are stored in memory after `data` will end with a
	// segmentation error at best.
}

impl Entry {
	pub(super) fn id(&self) -> StringId {
		self.id.unwrap_or_else(|| {
			debug_assert!(false, "Entry was not initialized");
			unsafe { hint::unreachable_unchecked() }
		})
	}

	pub(super) fn next(&self) -> &AtomicPtr<Self> {
		&self.next
	}

	pub(super) fn len(&self) -> u16 {
		self.len
	}

	pub(super) fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub(super) fn as_str(&self) -> &str {
		unsafe {
			let slice = slice::from_raw_parts(self.data.as_ptr(), self.len as usize);
			str::from_utf8_unchecked(slice)
		}
	}

	pub(super) fn iter(&self) -> impl Iterator<Item = &Self> {
		Entries {
			current: Some(self),
		}
	}
}

struct Entries<'a> {
	current: Option<&'a Entry>,
}

impl<'a> Iterator for Entries<'a> {
	type Item = &'a Entry;

	fn next(&mut self) -> Option<Self::Item> {
		self.current.map(|current| {
			let next = current.next().load(atomic::Ordering::Acquire);
			self.current = if next.is_null() {
				None
			} else {
				unsafe { Some(&*next) }
			};
			current
		})
	}
}
