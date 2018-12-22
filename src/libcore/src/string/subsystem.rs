// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, December 2018

use std::{
	fmt::{self, Debug, Formatter},
	hash::{BuildHasher, BuildHasherDefault, Hash, Hasher},
	str,
	sync::{
		atomic::{self, AtomicUsize, Ordering},
		Mutex,
	},
};

use astral_engine::third_party::slog::{info, o, Logger};

use crate::{hash::Murmur3, System};

use super::{Allocator, Entry, EntryHashTable, StaticRefVector, StringId};

#[cfg(feature = "track-strings")]
struct Tracker {
	used_memory: AtomicUsize,
	used_chunks: AtomicUsize,
	string_len: AtomicUsize,
	strings_allocated: AtomicUsize,
}

#[cfg(not(feature = "track-strings"))]
struct Tracker;

impl Tracker {
	#[cfg(feature = "track-strings")]
	fn new(memory: usize, allocations: usize) -> Self {
		Self {
			used_memory: memory.into(),
			used_chunks: allocations.into(),
			string_len: 0.into(),
			strings_allocated: 0.into(),
		}
	}

	#[cfg(not(feature = "track-strings"))]
	fn new(_memory: usize, _allocations: usize) -> Self {
		Tracker
	}

	#[cfg(feature = "track-strings")]
	fn add_memory(&self, memory: usize) {
		let _ = self
			.used_memory
			.fetch_add(memory, atomic::Ordering::Relaxed);
	}

	#[cfg(feature = "track-strings")]
	fn add_chunks(&self, chunks: usize) {
		let _ = self
			.used_chunks
			.fetch_add(chunks, atomic::Ordering::Relaxed);
	}

	#[cfg(feature = "track-strings")]
	fn add_allocations(&self, allocations: usize) {
		let _ = self
			.strings_allocated
			.fetch_add(allocations, atomic::Ordering::Relaxed);
	}

	#[cfg(feature = "track-strings")]
	fn add_len(&self, len: usize) {
		let _ = self.string_len.fetch_add(len, atomic::Ordering::Relaxed);
	}

	#[cfg(not(feature = "track-strings"))]
	fn add_memory(&self, _memory: usize) {}

	#[cfg(not(feature = "track-strings"))]
	fn add_chunks(&self, _chunks: usize) {}

	#[cfg(not(feature = "track-strings"))]
	fn add_allocations(&self, _allocations: usize) {}

	#[cfg(not(feature = "track-strings"))]
	fn add_len(&self, _len: usize) {}

	#[cfg(feature = "track-strings")]
	fn memory(&self) -> usize {
		self.used_memory.load(Ordering::Relaxed)
	}

	#[cfg(feature = "track-strings")]
	fn chunks(&self) -> usize {
		self.used_chunks.load(Ordering::Relaxed)
	}

	#[cfg(feature = "track-strings")]
	fn allocations(&self) -> usize {
		self.strings_allocated.load(Ordering::Relaxed)
	}

	#[cfg(feature = "track-strings")]
	fn average_length(&self) -> usize {
		if self.allocations() == 0 {
			0
		} else {
			self.string_len.load(Ordering::Relaxed) / self.allocations()
		}
	}
}

/// Manages optimized string allocation.
///
/// See the [module-level documentation] for more.
///
/// [module-level documentation]: index.html
pub struct Subsystem<H = BuildHasherDefault<Murmur3>> {
	log: Logger,
	allocator: Mutex<Allocator>,
	entry_hash_table: EntryHashTable,
	entry_reference_map: StaticRefVector<Entry>,
	build_hasher: H,
	tracker: Tracker,
}

impl Subsystem<BuildHasherDefault<Murmur3>> {
	/// Initialize the string subsystem from the given [core system] with the specified capacity
	/// for unique strings.
	///
	/// [core system]: astral_core::System
	///
	/// # Example
	///
	/// ```
	/// use astral::{
	/// 	Engine,
	/// 	core::{self, string},
	/// };
	/// # use astral::third_party::slog;
	///
	/// # let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	let engine = Engine::new(&logger);
	/// let core_system = core::System::new(&engine);
	/// # #[allow(unused_variables)]
	/// let string_subsystem = string::Subsystem::new(64, &core_system);
	/// ```
	pub fn new(max_strings: usize, system: &System) -> Self {
		let log = system.logger().new(o!("subsystem" => "string"));
		let (entry_hash_table, table_memory, table_chunks) = EntryHashTable::new();
		let (entry_reference_map, map_memory, map_chunks) = StaticRefVector::new(max_strings);
		info!(log, "initializing");
		Self {
			log,
			allocator: Mutex::new(Allocator::default()),
			entry_hash_table,
			entry_reference_map,
			build_hasher: BuildHasherDefault::default(),
			tracker: Tracker::new(table_memory + map_memory, table_chunks + map_chunks),
		}
	}
}

impl<H> Subsystem<H>
where
	H: BuildHasher,
{
	/// Initialize the string subsystem from the given [core system] with the specified capacity
	/// for unique strings and a hasher.
	///
	/// # Example
	///
	/// ```
	/// use std::hash::BuildHasherDefault;
	///
	/// # use astral::third_party::slog;
	/// use astral::{
	/// 	Engine,
	/// 	core::{
	///         self,
	///         hash::Murmur3,
	///         string::{self, Text},
	/// 	},
	/// };
	///
	/// # let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	let engine = Engine::new(&logger);
	/// let core_system = core::System::new(&engine);
	/// let string_subsystem = string::Subsystem::with_hasher(64, &core_system, BuildHasherDefault::<Murmur3>::default());
	///
	/// let text = Text::new("foo", &string_subsystem);
	/// assert_eq!(text, "foo");
	/// ```
	pub fn with_hasher(max_strings: usize, system: &System, hasher: H) -> Self {
		let log = system.logger().new(o!("subsystem" => "string"));
		let (entry_hash_table, table_memory, table_chunks) = EntryHashTable::new();
		let (entry_reference_map, map_memory, map_chunks) = StaticRefVector::new(max_strings);
		Self {
			log,
			allocator: Mutex::new(Allocator::default()),
			entry_hash_table,
			entry_reference_map,
			build_hasher: hasher,
			tracker: Tracker::new(table_memory + map_memory, table_chunks + map_chunks),
		}
	}

	pub(crate) fn create_string_id<T>(&self, string: T) -> StringId
	where
		T: AsRef<str>,
	{
		let string = string.as_ref();
		let mut hasher = self.build_hasher.build_hasher();
		Hash::hash_slice(string.as_bytes(), &mut hasher);
		let (id, memory, chunks, allocated) = self.entry_hash_table.find_or_insert(
			string,
			hasher.finish(),
			&self.entry_reference_map,
			&self.allocator,
			self.logger(),
		);
		self.tracker.add_memory(memory);
		self.tracker.add_chunks(chunks);
		if allocated {
			self.tracker.add_allocations(1);
			self.tracker.add_len(string.len());
		}
		debug_assert!(
			!self
				.entry_reference_map
				.get(id)
				.expect("Invalid string id")
				.is_null(),
			"Invalid pointer"
		);
		id
	}
}

impl<H> Subsystem<H> {
	/// Returns the used memory.
	///
	/// Requires the `track-strings` feature to be enabled.
	#[cfg(feature = "track-strings")]
	pub fn used_memory(&self) -> usize {
		self.tracker.memory()
	}

	/// Returns the used memory chunks.
	///
	/// Requires the `track-strings` feature to be enabled.
	#[cfg(feature = "track-strings")]
	pub fn allocations(&self) -> usize {
		self.tracker.chunks()
	}

	/// Returns the number of unique allocated strings.
	///
	/// Requires the `track-strings` feature to be enabled.
	#[cfg(feature = "track-strings")]
	pub fn strings_allocated(&self) -> usize {
		self.tracker.allocations()
	}

	/// Returns the average string length.
	///
	/// Requires the `track-strings` feature to be enabled.
	#[cfg(feature = "track-strings")]
	pub fn average_string_length(&self) -> usize {
		self.tracker.average_length()
	}

	/// Returns the logger of this string subsystem.
	///
	/// # Example
	///
	/// ```
	/// use astral::{
	/// 	Engine,
	/// 	core::{self, string},
	/// 	third_party::slog::info,
	/// };
	/// # use astral::third_party::slog;
	///
	/// # let logger = slog::Logger::root(slog::Discard, slog::o!());
	/// let engine = Engine::new(&logger);
	/// let core_system = core::System::new(&engine);
	/// let string_subsystem = string::Subsystem::new(64, &core_system);
	///
	/// info!(string_subsystem.logger(), "foo bar"; "additional" => "information");
	/// ```
	pub fn logger(&self) -> &Logger {
		&self.log
	}

	pub(super) fn string(&self, id: StringId) -> &str {
		debug_assert!(
			!self
				.entry_reference_map
				.get(id)
				.expect("Invalid string id")
				.is_null(),
			"Index is null"
		);
		unsafe { (*self.entry_reference_map.get_unchecked(id)).as_str() }
	}

	pub(super) fn is_empty(&self, id: StringId) -> bool {
		debug_assert!(
			!self
				.entry_reference_map
				.get(id)
				.expect("Invalid string id")
				.is_null(),
			"Index is null"
		);
		unsafe { (*self.entry_reference_map.get_unchecked(id)).is_empty() }
	}

	pub(super) fn len(&self, id: StringId) -> usize {
		debug_assert!(
			!self
				.entry_reference_map
				.get(id)
				.expect("Invalid string id")
				.is_null(),
			"Index is null"
		);
		unsafe { (*self.entry_reference_map.get_unchecked(id)).len() as usize }
	}
}

impl<H> Debug for Subsystem<H> {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		let mut debug = fmt.debug_struct("Subsystem");
		#[cfg(feature = "track-strings")]
		{
			let _ = debug
				.field("strings_allocated", &self.strings_allocated())
				.field("used_memory", &self.used_memory())
				.field("allocations", &self.allocations())
				.field("average_string_length", &self.average_string_length());
		}
		debug.finish()
	}
}

impl<H> Drop for Subsystem<H> {
	fn drop(&mut self) {
		#[cfg(feature = "track-strings")]
		info!(self.logger(), "shutting down";
			"strings" => self.strings_allocated(),
			"memory" => self.used_memory(),
			"allocations" => self.allocations(),
			"average_string_length" => self.average_string_length(),
		);
		#[cfg(not(feature = "track-strings"))]
		info!(self.logger(), "shutting down");
	}
}

unsafe impl<H> Send for Subsystem<H> {}
unsafe impl<H> Sync for Subsystem<H> {}
