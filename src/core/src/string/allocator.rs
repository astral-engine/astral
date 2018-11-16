// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::{
	alloc::{GlobalAlloc, Layout, System},
	mem, ptr,
};

use super::{Entry, DATA_OFFSET, PAGE_SIZE};

/// Allocates Entries from a pool.
///
/// The allocated Entries will never be dropped.
// TODO(#8): Make fields private
pub struct Allocator {
	pub(super) current_pool_start: *mut u8,
	pub(super) current_pool_end: *mut u8,
}

impl Allocator {
	/// Constructs a new `Allocator`.
	// TODO(#8): make const
	pub fn new() -> Self {
		Self {
			current_pool_start: ptr::null_mut(),
			current_pool_end: ptr::null_mut(),
		}
	}

	fn allocate_page(&mut self) {
		debug_assert!(
			PAGE_SIZE >= mem::size_of::<Entry>(),
			"PAGE_SIZE must be at least as large as Entry. PAGE_SIZE is {}, but Entry is {} in size.", PAGE_SIZE, mem::size_of::<Entry>()
		);
		unsafe {
			let layout = Layout::from_size_align_unchecked(
				PAGE_SIZE,
				mem::align_of::<Entry>(),
			);
			self.current_pool_start = System.alloc(layout);
			self.current_pool_end = self.current_pool_start.add(PAGE_SIZE)
		}
	}

	fn capacity(&self) -> usize {
		self.current_pool_end as usize - self.current_pool_start as usize
	}

	#[cfg(not(unstable))]
	fn aligned_offset(&self) -> usize {
		let addr = self.current_pool_start as usize;
		let remainder = addr % mem::align_of::<Entry>();
		if remainder == 0 {
			0
		} else {
			mem::align_of::<Entry>() - remainder
		}
	}

	#[cfg(unstable)]
	fn aligned_offset(&self) -> usize {
		self.current_pool_start
			.align_offset(mem::align_of::<Entry>())
	}

	/// Allocates a new entry and sets the `index` to 0.
	// TODO(#7): Use tool-lints
	#[cfg_attr(
		feature = "cargo-clippy",
		allow(cast_possible_truncation, cast_ptr_alignment)
	)]
	pub fn allocate(&mut self, string: &str) -> *mut Entry {
		let len = string.len();
		if self.capacity() < len + DATA_OFFSET {
			self.allocate_page();
		}
		debug_assert_eq!(self.aligned_offset(), 0);

		unsafe {
			let entry = &mut *(self.current_pool_start as *mut Entry);
			self.current_pool_start =
				self.current_pool_start.add(len + DATA_OFFSET);
			self.current_pool_start =
				self.current_pool_start.add(self.aligned_offset());
			entry.index = None;
			entry.len = len as u16;
			ptr::copy_nonoverlapping(
				string.as_ptr(),
				entry.data.as_mut_ptr(),
				string.len(),
			);
			entry
		}
	}
}

impl Default for Allocator {
	fn default() -> Self {
		Self::new()
	}
}
