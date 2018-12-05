// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::{
	mem,
	num::NonZeroU32,
	sync::{
		atomic::{self, AtomicPtr},
		Mutex,
	},
};

use super::{Entry, ENTRY_REFERENCE_MAP, USED_MEMORY, USED_MEMORY_CHUNKS};

const NUM_BUCKETS: usize = 64 * 1024;

/// A hash table which stores pointers to `Entry`.
pub struct EntryHashTable {
	head: Box<[AtomicPtr<Entry>; NUM_BUCKETS]>,
	mutex: Mutex<()>,
}

impl EntryHashTable {
	/// Constructs a new hash table.
	pub fn new() -> Self {
		USED_MEMORY.fetch_add(
			mem::size_of::<AtomicPtr<Entry>>() * NUM_BUCKETS,
			atomic::Ordering::Acquire,
		);
		USED_MEMORY_CHUNKS.fetch_add(1, atomic::Ordering::Acquire);
		Self {
			head: Box::new(unsafe { mem::zeroed() }),
			mutex: Mutex::default(),
		}
	}

	/// Searches the table for an entry with the given name and hash.
	/// Returns [`None`] if no entry was found.
	#[allow(clippy::cast_possible_truncation)]
	pub fn find(&self, name: &str, hash: u16) -> Option<&Entry> {
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

	/// Searches the table for an entry with the given name and hash or insers
	/// a new one, if none is found.
	#[allow(clippy::cast_possible_truncation)]
	pub fn find_or_insert(&self, name: &str, hash: u64) -> NonZeroU32 {
		let hash = hash as u16;

		if let Some(entry) = self.find(name, hash) {
			return entry.index();
		}

		let _guard = self.mutex.lock().unwrap();
		if let Some(entry) = self.find(name, hash) {
			return entry.index();
		}
		unsafe {
			let entry = Entry::allocate(name);
			(*entry).index = Some(ENTRY_REFERENCE_MAP.push(&*entry));
			let head = self.head[hash as usize].load(atomic::Ordering::Relaxed);
			if head.is_null() {
				self.head[hash as usize]
					.store(entry, atomic::Ordering::Release);
			} else {
				(*head)
					.iter()
					.last()
					.expect("unexpeted end of hash bucket")
					.next()
					.store(entry, atomic::Ordering::Release)
			}
			(*entry).index()
		}
	}
}

impl Default for EntryHashTable {
	fn default() -> Self {
		Self::new()
	}
}

unsafe impl Send for EntryHashTable {}
unsafe impl Sync for EntryHashTable {}
