// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::{
	alloc::{GlobalAlloc, Layout, System},
	mem, ptr,
	sync::atomic,
};

use super::{Entry, ALLOCATED_STRINGS, DATA_OFFSET, PAGE_SIZE, USED_MEMORY, USED_MEMORY_CHUNKS};

/// Allocates Entries from a pool.
///
/// The allocated Entries will never be dropped.
pub(super) struct Allocator {
	current_pool_start: *mut u8,
	current_pool_end: *mut u8,
}

impl Allocator {
	/// Constructs a new `Allocator`.
	pub(super) const fn new() -> Self {
		Self {
			current_pool_start: ptr::null_mut(),
			current_pool_end: ptr::null_mut(),
		}
	}

	fn allocate_page(&mut self) {
		debug_assert!(
			PAGE_SIZE >= mem::size_of::<Entry>(),
			"PAGE_SIZE must be at least as large as Entry. PAGE_SIZE is {}, but Entry is {} in \
			 size.",
			PAGE_SIZE,
			mem::size_of::<Entry>()
		);
		let _ = USED_MEMORY.fetch_add(PAGE_SIZE, atomic::Ordering::Acquire);
		let _ = USED_MEMORY_CHUNKS.fetch_add(1, atomic::Ordering::Acquire);
		unsafe {
			let layout = Layout::from_size_align_unchecked(PAGE_SIZE, mem::align_of::<Entry>());
			self.current_pool_start = System.alloc(layout);
			self.current_pool_end = self.current_pool_start.add(PAGE_SIZE)
		}
	}

	fn capacity(&self) -> usize {
		self.current_pool_end as usize - self.current_pool_start as usize
	}

	#[cfg(not(unstable))]
	// ToDo(#3): Use `align_offset`
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
	#[allow(clippy::cast_possible_truncation, clippy::cast_ptr_alignment)]
	pub(super) fn allocate(&mut self, string: &str) -> *mut Entry {
		let len = string.len();
		if self.capacity() < len + DATA_OFFSET {
			self.allocate_page();
		}
		debug_assert_eq!(self.aligned_offset(), 0);
		let _ = ALLOCATED_STRINGS.fetch_add(1, atomic::Ordering::Acquire);

		unsafe {
			let entry = &mut *(self.current_pool_start as *mut Entry);
			self.current_pool_start = self.current_pool_start.add(len + DATA_OFFSET);
			self.current_pool_start = self.current_pool_start.add(self.aligned_offset());
			entry.index = None;
			entry.len = len as u16;
			ptr::copy_nonoverlapping(string.as_ptr(), entry.data.as_mut_ptr(), string.len());
			entry
		}
	}
}

impl Default for Allocator {
	fn default() -> Self {
		Self::new()
	}
}
