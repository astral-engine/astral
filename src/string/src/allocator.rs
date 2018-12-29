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
	alloc::{GlobalAlloc, Layout, System},
	mem,
	ptr,
};

use super::{Entry, DATA_OFFSET, PAGE_SIZE};

/// Allocates Entries from a pool.
pub(super) struct Allocator {
	current_pool_start: *mut u8,
	current_pool_end: *mut u8,
	pools: Vec<*mut u8>,
}

impl Allocator {
	/// Constructs a new `Allocator`.
	pub(super) fn new() -> Self {
		Self {
			current_pool_start: ptr::null_mut(),
			current_pool_end: ptr::null_mut(),
			pools: Vec::default(),
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
		unsafe {
			let layout = Layout::from_size_align_unchecked(PAGE_SIZE, mem::align_of::<Entry>());
			self.current_pool_start = System.alloc_zeroed(layout);
			self.current_pool_end = self.current_pool_start.add(PAGE_SIZE);
		}
		self.pools.push(self.current_pool_start);
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

	#[allow(clippy::cast_possible_truncation, clippy::cast_ptr_alignment)]
	pub(super) fn allocate(&mut self, string: &str) -> (&mut Entry, usize, usize) {
		let len = string.len();
		let (memory, chunks) = if self.capacity() < len + DATA_OFFSET {
			self.allocate_page();
			(PAGE_SIZE, 1)
		} else {
			(0, 0)
		};

		unsafe {
			let entry = &mut *(self.current_pool_start as *mut Entry);
			self.current_pool_start = self.current_pool_start.add(len + DATA_OFFSET);
			self.current_pool_start = self.current_pool_start.add(self.aligned_offset());
			entry.id = None;
			entry.len = len as u16;
			ptr::copy_nonoverlapping(string.as_ptr(), entry.data.as_mut_ptr(), len);
			(&mut *entry, memory, chunks)
		}
	}
}

impl Drop for Allocator {
	fn drop(&mut self) {
		for pool in &self.pools {
			unsafe {
				System.dealloc(
					*pool,
					Layout::from_size_align_unchecked(PAGE_SIZE, mem::align_of::<Entry>()),
				);
			}
		}
	}
}

impl Default for Allocator {
	fn default() -> Self {
		Self::new()
	}
}
