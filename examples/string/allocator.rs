// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use super::{Entry, EntryData};

use std::{
	alloc::{GlobalAlloc, Layout, System},
	mem,
	ptr,
	usize,
};

const MINIMUM_PAGE_SIZE: usize = 1024 * 64;

/// Allocates Entries from a pool.
pub(super) struct Allocator {
	entries: Vec<EntryData>,
	buffers: Vec<String>,
}

impl Allocator {
	// Constructs a new `Allocator`.
	pub(super) fn new() -> Self {
		Self {
			entries: Vec::default(),
			// 		current_pool_start: ptr::null_mut(),
			// 		current_pool_end: ptr::null_mut(),
			buffers: Vec::default(),
		}
	}

	fn allocate_page(&mut self, capacity: usize) {
		// self.pools.push(String)
		// 	unsafe {
		// 		let layout = Layout::from_size_align_unchecked(PAGE_SIZE, mem::align_of::<Entry>());
		// 		self.current_pool_start = System.alloc(layout);
		// 		self.current_pool_end = self.current_pool_start.add(PAGE_SIZE);
		// 		self.pools.push(self.current_pool_start);
		// 	}
	}

	// pub(super) fn capacity(&self) -> usize {
	// 	self.current_pool_end as usize - self.current_pool_start as usize
	// }

	// #[cfg(not(unstable))]
	// // ToDo(#3): Use `align_offset`
	// fn aligned_offset(&self) -> usize {
	// 	let addr = self.current_pool_start as usize;
	// 	let remainder = addr % mem::align_of::<Entry>();
	// 	if remainder == 0 {
	// 		0
	// 	} else {
	// 		mem::align_of::<Entry>() - remainder
	// 	}
	// }

	// #[cfg(unstable)]
	// fn aligned_offset(&self) -> usize {
	// 	self.current_pool_start
	// 		.align_offset(mem::align_of::<Entry>())
	// }

	// #[allow(clippy::cast_possible_truncation, clippy::cast_ptr_alignment)]
	pub(super) fn allocate(&mut self, string: &str) -> Entry {
		let buffer = if let Some(buffer) = self.buffers.last_mut() {
			buffer
		} else {
			self.buffers.push(String::with_capacity(usize::max(
				string.len(),
				MINIMUM_PAGE_SIZE,
			)));
			self.buffers.last_mut().unwrap_or_else(|| unreachable!())
		};
		buffer.push_str(string);
		// 	let len = string.len();
		// 	if self.capacity() < len + DATA_OFFSET {
		// 		self.allocate_page();
		// 	}
		// 	debug_assert_eq!(self.aligned_offset(), 0);

		// 	unsafe {
		// 		let entry = &mut *(self.current_pool_start as *mut Entry);
		// 		self.current_pool_start = self.current_pool_start.add(len + DATA_OFFSET);
		// 		self.current_pool_start = self.current_pool_start.add(self.aligned_offset());
		// 		entry.index = None;
		// 		entry.len = len as u16;
		// 		ptr::copy_nonoverlapping(string.as_ptr(), entry.data.as_mut_ptr(), string.len());
		// 		entry
		// 	}
		unimplemented!()
	}
}

// impl Drop for Allocator {
// 	fn drop(&mut self) {
// 		for pool in &self.pools {
// 			unsafe {
// 				System.dealloc(
// 					*pool,
// 					Layout::from_size_align_unchecked(PAGE_SIZE, mem::align_of::<Entry>()),
// 				);
// 			}
// 		}
// 	}
// }

// impl Default for Allocator {
// 	fn default() -> Self {
// 		Self::new()
// 	}
// }
