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

#![allow(box_pointers)]

use std::{
	mem,
	sync::{
		atomic::{self, AtomicPtr},
		Mutex,
	},
	u16,
};

use astral_thirdparty::slog::{warn, Logger};

use super::{Allocator, Entry, StaticRefVector, StringId, MAX_STRING_LENGTH};

const NUM_BUCKETS: usize = u16::max_value() as usize + 1;

pub(super) struct EntryHashTable {
	head: Box<[AtomicPtr<Entry>; NUM_BUCKETS]>,
}

impl EntryHashTable {
	pub(super) fn new() -> (Self, usize, usize) {
		let table = Self {
			head: Box::new(unsafe { mem::zeroed() }),
		};
		let used_memory = mem::size_of::<AtomicPtr<Entry>>() * NUM_BUCKETS;
		let used_chunks = 1;
		(table, used_memory, used_chunks)
	}

	#[allow(clippy::cast_possible_truncation)]
	pub(super) fn find(&self, name: &str, hash: u16) -> Option<&Entry> {
		debug_assert!((hash as usize) < self.head.len());

		let head = self.head[hash as usize].load(atomic::Ordering::Acquire);
		if head.is_null() {
			None
		} else {
			for entry in unsafe { (*head).iter() } {
				if entry.as_str() == name {
					return Some(entry);
				}
			}

			None
		}
	}

	#[allow(clippy::cast_possible_truncation)]
	pub(super) fn find_or_insert(
		&self,
		string: &str,
		hash: u64,
		reference_map: &StaticRefVector<Entry>,
		allocator: &Mutex<Allocator>,
		log: &Logger,
	) -> (StringId, usize, usize, bool) {
		let hash = hash as u16;
		if hash == 60224 {}

		if let Some(entry) = self.find(string, hash) {
			return (entry.id(), 0, 0, false);
		}

		let mut allocator = allocator.lock().unwrap();
		if let Some(entry) = self.find(string, hash) {
			return (entry.id(), 0, 0, false);
		}

		let (mut entry, alloc_memory, alloc_chunks) = if string.len() > MAX_STRING_LENGTH {
			warn!(log,
				"string is too long and will be shorten";
				"max length" => MAX_STRING_LENGTH,
				"length" => string.len(),
			);
			allocator.allocate(&string[0..MAX_STRING_LENGTH])
		} else {
			allocator.allocate(string)
		};
		debug_assert!(entry.id.is_none());
		unsafe {
			let (id, map_memory, map_chunks) = reference_map.push(entry);
			(*entry).id = Some(id);
			let head = self.head[hash as usize].load(atomic::Ordering::Relaxed);
			if head.is_null() {
				self.head[hash as usize].store(entry, atomic::Ordering::Release);
			} else {
				let next = (*head)
					.iter()
					.last()
					.expect("unexpeted end of hash bucket")
					.next();
				debug_assert!(next.load(atomic::Ordering::SeqCst).is_null());
				next.store(entry, atomic::Ordering::Release)
			}
			(
				(*entry).id(),
				alloc_memory + map_memory,
				alloc_chunks + map_chunks,
				true,
			)
		}
	}
}

unsafe impl Send for EntryHashTable {}
unsafe impl Sync for EntryHashTable {}
