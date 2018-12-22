// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

mod drain;
mod drain_filter;
mod into_iter;
mod iter;
mod iter_mut;
mod keys;
mod slot;
mod values;
mod values_mut;

pub(super) use self::{
	drain::Drain,
	drain_filter::DrainFilter,
	into_iter::IntoIter,
	iter::Iter,
	iter_mut::IterMut,
	keys::Keys,
	values::Values,
	values_mut::ValuesMut,
};

use std::{
	fmt::{self, Debug, Formatter},
	ops::{Index, IndexMut},
};

use crate::math::num::{AsPrimitive, PrimUnsignedInt};

use super::Key;

use self::slot::Slot;

/// A storage with stable unique keys.
///
/// See [module documentation](index.html) for more details.
pub struct SlotMap<T, Idx = u32>
where
	Idx: PrimUnsignedInt + AsPrimitive<usize>,
	usize: AsPrimitive<Idx>,
{
	slots: Vec<Slot<T, Idx>>,
	free_head: Idx,
	len: Idx,
}

impl<T, Idx> SlotMap<T, Idx>
where
	Idx: PrimUnsignedInt + AsPrimitive<usize>,
	usize: AsPrimitive<Idx>,
{
	/// Construct a new, empty `SparseSlotMap`.
	///
	/// The slot map will not allocate until values are inserted.
	///
	/// # Example
	///
	/// ```
	/// use astral::core::collections::SparseSlotMap;
	///
	/// # #[allow(unused_variables)]
	/// let map: SparseSlotMap<i32> = SparseSlotMap::new();
	/// ```
	pub fn new() -> Self {
		Self::with_capacity(0)
	}

	/// Construct a new, empty `SparseSlotMap` with the specified capacity.
	///
	/// The slot map will be able to hold exactly `capacity` elements without
	/// reallocating. If `capacity` is 0, the vector will not allocate.
	///
	/// # Example
	///
	/// ```
	/// use astral::core::collections::SparseSlotMap;
	///
	/// let mut map: SparseSlotMap<i32> = SparseSlotMap::with_capacity(10);
	///
	/// // The slot map contains no items, even though it has capacity for more
	/// assert_eq!(map.len(), 0);
	///
	/// // These are all done without reallocating...
	/// for i in 0..10 {
	///     map.insert(i);
	/// }
	///
	/// // ...but this may make the slot map reallocate
	/// map.insert(11);
	/// ```
	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			slots: Vec::with_capacity(capacity),
			free_head: Idx::zero(),
			len: Idx::zero(),
		}
	}

	/// Returns the number of elements the slot map can hold without
	/// reallocating.
	///
	/// # Examples
	///
	/// ```
	/// use astral::core::collections::SparseSlotMap;
	///
	/// let map: SparseSlotMap<i32> = SparseSlotMap::with_capacity(10);
	/// assert_eq!(map.capacity(), 10);
	/// ```
	pub fn capacity(&self) -> usize {
		self.slots.capacity()
	}

	/// Reserves capacity for at least `additional` more elements to be inserted
	/// in the given slot map. The collection may reserve more space to avoid
	/// frequent reallocations. After calling `reserve`, capacity will be
	/// greater than or equal to `self.len() + additional`. Does nothing if
	/// capacity is already sufficient.
	///
	/// # Panics
	///
	/// Panics if the new capacity overflows `usize`.
	///
	/// # Examples
	///
	/// ```
	/// use astral::core::collections::SparseSlotMap;
	///
	/// let mut map: SparseSlotMap<i32> = SparseSlotMap::with_capacity(1);
	/// map.insert(1);
	///
	/// map.reserve(10);
	/// assert!(map.capacity() >= 11);
	/// ```
	pub fn reserve(&mut self, additional: usize) {
		let len: usize = self.len().as_();
		let needed: usize = (len + additional).saturating_sub(self.slots.len());
		self.slots.reserve(needed)
	}

	/// Returns the number of elements in the slot map, also referred to
	/// as its 'length'.
	///
	/// # Examples
	///
	/// ```
	/// use astral::core::collections::SparseSlotMap;
	///
	/// let mut map: SparseSlotMap<i32> = SparseSlotMap::with_capacity(3);
	///
	/// for i in 0..3 {
	///     map.insert(i);
	/// }
	///
	/// assert_eq!(map.len(), 3);
	/// ```
	pub fn len(&self) -> Idx {
		self.len
	}

	/// Returns `true` if the slot map contains no elements.
	///
	/// # Examples
	///
	/// ```
	/// use astral::core::collections::SparseSlotMap;
	///
	/// let mut map: SparseSlotMap<i32> = SparseSlotMap::with_capacity(1);
	///
	/// assert!(map.is_empty());
	///
	/// map.insert(1);
	/// assert!(!map.is_empty());
	/// ```
	pub fn is_empty(&self) -> bool {
		self.len() == Idx::zero()
	}

	/// Creates a new key which can be used later.
	///
	/// # Panics
	///
	/// Panics if the number of elements in the slot map overflows `Idx`.
	///
	/// # Example
	///
	/// ```
	/// # fn main() -> Result<(), u32> {
	/// use astral::core::collections::SparseSlotMap;
	///
	/// let mut map: SparseSlotMap<u32> = SparseSlotMap::with_capacity(2);
	/// let key1 = map.create_key();
	/// let key2 = map.create_key();
	///
	/// assert!(map.is_empty());
	///
	/// map.insert_with_key(key2, 200)?;
	/// map.insert_with_key(key1, 100)?;
	/// assert_eq!(map[key1], 100);
	/// assert_eq!(map[key2], 200);
	/// # Ok(()) }
	/// ```
	pub fn create_key(&mut self) -> Key<Idx> {
		let idx = self.free_head;
		let len = self.slots.len();

		if let Some(slot) = self.slots.get_mut(idx.as_()) {
			self.free_head = slot.index();
			return Key::new(idx, slot.version());
		}
		assert_ne!(
			len,
			Idx::max_value().as_(),
			"number of elements overflows `Idx`"
		);
		self.slots.push(Slot::new());
		self.free_head = 1.as_() + len.as_();

		Key::new(idx, Idx::one())
	}

	/// Returns if a key is stored in the map.
	///
	/// # Example
	///
	/// ```
	/// use astral::core::collections::SparseSlotMap;
	///
	/// let mut map: SparseSlotMap<u32> = SparseSlotMap::new();
	/// let key1 = map.insert(100);
	/// assert!(map.contains_key(key1));
	/// ```
	///
	/// A key returned from `create_key()` is not contained in the map
	/// until inserted with `insert_with_key()`
	///
	/// ```
	/// # fn main() -> Result<(), u32> {
	/// # use astral::core::collections::SparseSlotMap;
	/// # let mut map: SparseSlotMap<u32> = SparseSlotMap::new();
	///	let key2 = map.create_key();
	/// assert!(!map.contains_key(key2));
	/// map.insert_with_key(key2, 200)?;
	/// assert!(map.contains_key(key2));
	/// # Ok(()) }
	/// ```
	pub fn contains_key(&self, key: Key<Idx>) -> bool {
		self.slots.get(key.index().as_()).map_or(false, |slot| {
			slot.occupied() && slot.version() == key.version()
		})
	}

	/// Inserts a value into the map returning the key.
	///
	/// # Panics
	///
	/// Panics if the number of elements in the slot map overflows `Idx`.
	///
	/// # Example
	///
	/// ```
	/// use astral::core::collections::SparseSlotMap;
	///
	/// let mut map: SparseSlotMap<u32> = SparseSlotMap::new();
	/// let key1 = map.insert(100);
	/// assert_eq!(map[key1], 100);
	/// map.remove(key1);
	/// assert!(!map.contains_key(key1));
	/// ```
	pub fn insert(&mut self, value: T) -> Key<Idx> {
		let key = self.create_key();
		let _ = self.insert_with_key(key, value);
		key
	}

	/// Inserts a value at the given position. The key has to be created with
	/// `create_key`. It returns the previously stored value if any.
	///
	/// # Errors
	///
	/// Returns back the passed value if the key is not valid.
	///
	/// # Example
	///
	/// ```
	/// use astral::core::collections::SparseSlotMap;
	///
	/// let mut map: SparseSlotMap<u32> = SparseSlotMap::new();
	/// let key = map.create_key();
	///
	/// assert!(map.insert_with_key(key, 100).unwrap().is_none());
	/// assert_eq!(map[key], 100);
	/// ```
	///
	/// If the key is used again, the value will be overwritten:
	/// ```
	/// # use astral::core::collections::SparseSlotMap;
	/// # let mut map: SparseSlotMap<u32> = SparseSlotMap::new();
	/// # let key = map.insert(100);
	/// assert_eq!(map.insert_with_key(key, 200), Ok(Some(100)));
	/// assert_eq!(map[key], 200);
	/// ```
	///
	/// If the key is not valid, the value will be passed back:
	/// ```
	/// # use astral::core::collections::SparseSlotMap;
	/// # let mut map: SparseSlotMap<u32> = SparseSlotMap::new();
	/// # let key = map.insert(200);
	/// map.remove(key);
	/// assert_eq!(map.insert_with_key(key, 300), Err(300));
	/// ```
	pub fn insert_with_key(&mut self, key: Key<Idx>, value: T) -> Result<Option<T>, T> {
		if let Some(slot) = self.slots.get_mut(key.index().as_()) {
			if key.version() != slot.version() {
				return Err(value);
			}

			if !slot.occupied() {
				self.len += Idx::one();
				if !slot.reserved() {
					self.free_head = slot.index();
				}
			}
			Ok(slot.set_value(value))
		} else {
			Err(value)
		}
	}

	/// Removes the value at the given key
	///
	/// # Example
	///
	/// ```
	/// use astral::core::collections::SparseSlotMap;
	///
	/// let mut map: SparseSlotMap<u32> = SparseSlotMap::new();
	/// let key = map.insert(100);
	/// assert_eq!(map.len(), 1);
	/// map.remove(key);
	/// assert!(map.is_empty());
	/// ```
	///
	/// Keys, which are created with `create_key()` can be discarded with
	/// this function as well
	///
	/// ```
	/// # use astral::core::collections::SparseSlotMap;
	///
	/// # let mut map: SparseSlotMap<u32> = SparseSlotMap::new();
	/// let key = map.create_key();
	/// assert!(map.is_empty());
	/// map.remove(key);
	/// assert!(map.is_empty());
	/// ```
	pub fn remove(&mut self, key: Key<Idx>) -> Option<T> {
		if let Some(slot) = self.slots.get_mut(key.index().as_()) {
			if slot.version() != key.version() {
				return None;
			}
			if slot.occupied() || slot.reserved() {
				if slot.occupied() {
					self.len -= Idx::one();
				}
				slot.increment_version();
				let value = slot.set_index(self.free_head);
				self.free_head = key.index();
				value
			} else {
				None
			}
		} else {
			None
		}
	}

	/// Clears the slot map. Keeps the allocated memory for reuse.
	///
	/// This function must iterate over all slots, empty or not. In the face of
	/// many deleted elements it can be inefficient.
	///
	/// # Examples
	///
	/// ```
	/// use astral::core::collections::SparseSlotMap;
	///
	/// let mut map: SparseSlotMap<i32> = SparseSlotMap::with_capacity(10);
	///
	/// for i in 0..10 {
	///     map.insert(i);
	/// }
	///
	/// assert_eq!(map.len(), 10);
	/// map.clear();
	/// assert!(map.is_empty());
	/// ```
	pub fn clear(&mut self) {
		let _ = self.drain();
	}

	/// Retains only the elements specified by the predicate.
	///
	/// In other words, remove all key-value pairs (k, v) such that
	/// `f(k, &mut v)` returns false.
	///
	/// This function must iterate over all slots, empty or not. In the face of
	/// many deleted elements it can be inefficient.
	///
	/// # Examples
	///
	/// ```
	/// use astral::core::collections::SparseSlotMap;
	///
	/// let mut map: SparseSlotMap<i32> = SparseSlotMap::with_capacity(4);
	///
	/// let k1 = map.insert(1);
	/// let k2 = map.insert(2);
	/// # #[allow(unused_variables)]
	/// let k3 = map.insert(3);
	/// let k4 = map.insert(4);
	///
	/// map.retain(|key, val| key == k1 || *val % 2 == 0);
	/// assert_eq!(map.into_iter().collect::<Vec<_>>(), vec![(k1, 1), (k2, 2), (k4, 4)]);
	/// ```
	pub fn retain<F>(&mut self, mut predicate: F)
	where
		F: FnMut(Key<Idx>, &mut T) -> bool,
	{
		let _ = self.drain_filter(|key, value| !predicate(key, value));
	}

	/// Returns a reference to the value corresponding to the key.
	///
	/// # Examples
	///
	/// ```
	/// use astral::core::collections::SparseSlotMap;
	///
	/// let mut map: SparseSlotMap<i32> = SparseSlotMap::with_capacity(1);
	///
	/// let key = map.insert(10);
	/// assert_eq!(map.get(key), Some(&10));
	/// map.remove(key);
	/// assert_eq!(map.get(key), None);
	/// ```
	pub fn get(&self, key: Key<Idx>) -> Option<&T> {
		self.slots
			.get(key.index().as_())
			.filter(|slot| slot.occupied() && slot.version() == key.version())
			.map(|slot| slot.value())
	}

	/// Returns a mutable reference to the value corresponding to the key.
	///
	/// # Examples
	///
	/// ```
	/// use astral::core::collections::SparseSlotMap;
	///
	/// let mut map: SparseSlotMap<f32> = SparseSlotMap::with_capacity(1);
	///
	/// let key = map.insert(3.5);
	/// if let Some(x) = map.get_mut(key) {
	///     *x += 3.0;
	/// }
	/// assert_eq!(map[key], 6.5);
	/// ```
	pub fn get_mut(&mut self, key: Key<Idx>) -> Option<&mut T> {
		self.slots
			.get_mut(key.index().as_())
			.filter(|slot| slot.occupied() && slot.version() == key.version())
			.map(|slot| slot.value_mut())
	}

	/// An iterator visiting all key-value pairs in arbitrary order. The
	/// iterator element type is `(Key, &'a T)`.
	///
	/// This function must iterate over all slots, empty or not. In the face of
	/// many deleted elements it can be inefficient.
	///
	/// # Examples
	///
	/// ```
	/// use astral::core::collections::SparseSlotMap;
	///
	/// let mut map: SparseSlotMap<i32> = SparseSlotMap::with_capacity(3);
	///
	/// let k0 = map.insert(0);
	/// let k1 = map.insert(1);
	/// let k2 = map.insert(2);
	///
	/// let mut it = map.iter();
	/// assert_eq!(it.next(), Some((k0, &0)));
	/// assert_eq!(it.len(), 2);
	/// assert_eq!(it.next(), Some((k1, &1)));
	/// assert_eq!(it.next(), Some((k2, &2)));
	/// assert_eq!(it.next(), None);
	/// ```
	pub fn iter(&self) -> Iter<'_, T, Idx> {
		Iter {
			slots: self.slots.iter().enumerate(),
			num_left: self.len(),
		}
	}

	/// An iterator visiting all key-value pairs in arbitrary order, with
	/// mutable references to the values. The iterator element type is
	/// `(Key, &'a mut T)`.
	///
	/// This function must iterate over all slots, empty or not. In the face of
	/// many deleted elements it can be inefficient.
	///
	/// # Examples
	///
	/// ```
	/// use astral::core::collections::SparseSlotMap;
	///
	/// let mut map: SparseSlotMap<i32> = SparseSlotMap::with_capacity(3);
	///
	/// # #[allow(unused_variables)]
	/// let k0 = map.insert(10);
	/// let k1 = map.insert(20);
	/// # #[allow(unused_variables)]
	/// let k2 = map.insert(30);
	///
	/// for (k, v) in map.iter_mut() {
	///     if k != k1 {
	///         *v *= -1;
	///     }
	/// }
	///
	/// assert_eq!(map.values().collect::<Vec<_>>(), vec![&-10, &20, &-30]);
	/// ```
	pub fn iter_mut(&mut self) -> IterMut<'_, T, Idx> {
		let num_left = self.len();
		IterMut {
			slots: self.slots.iter_mut().enumerate(),
			num_left,
		}
	}

	/// An iterator visiting all keys in arbitrary order. The iterator element
	/// type is `Key`.
	///
	/// This function must iterate over all slots, empty or not. In the face of
	/// many deleted elements it can be inefficient.
	///
	/// # Examples
	///
	/// ```
	/// use astral::core::collections::SparseSlotMap;
	///
	/// let mut map: SparseSlotMap<i32> = SparseSlotMap::with_capacity(3);
	///
	/// let k0 = map.insert(10);
	/// let k1 = map.insert(20);
	/// let k2 = map.insert(30);
	/// let v: Vec<_> = map.keys().collect();
	/// assert_eq!(v, vec![k0, k1, k2]);
	/// ```
	pub fn keys(&self) -> Keys<'_, T, Idx> {
		Keys(self.iter())
	}

	/// An iterator visiting all values in arbitrary order. The iterator element
	/// type is `&'a T`.
	///
	/// This function must iterate over all slots, empty or not. In the face of
	/// many deleted elements it can be inefficient.
	///
	/// # Examples
	///
	/// ```
	/// use astral::core::collections::SparseSlotMap;
	///
	/// let mut map: SparseSlotMap<i32> = SparseSlotMap::with_capacity(3);
	///
	/// map.insert(10);
	/// map.insert(20);
	/// map.insert(30);
	/// let v: Vec<_> = map.values().collect();
	/// assert_eq!(v, vec![&10, &20, &30]);
	/// ```
	pub fn values(&self) -> Values<'_, T, Idx> {
		Values(self.iter())
	}

	/// An iterator visiting all values mutably in arbitrary order. The iterator
	/// element type is `&'a mut T`.
	///
	/// This function must iterate over all slots, empty or not. In the face of
	/// many deleted elements it can be inefficient.
	///
	/// # Examples
	///
	/// ```
	/// use astral::core::collections::SparseSlotMap;
	///
	/// let mut map: SparseSlotMap<i32> = SparseSlotMap::with_capacity(3);
	///
	/// map.insert(10);
	/// map.insert(20);
	/// map.insert(30);
	/// map.values_mut().for_each(|n| { *n *= 3 });
	/// let v: Vec<_> = map.into_iter().map(|(_k, v)| v).collect();
	/// assert_eq!(v, vec![30, 60, 90]);
	/// ```
	pub fn values_mut(&mut self) -> ValuesMut<'_, T, Idx> {
		ValuesMut(self.iter_mut())
	}

	/// Creates a draining iterator that yields the removed items.
	///
	/// # Examples
	///
	/// ```
	/// use astral::core::collections::SparseSlotMap;
	///
	/// let mut map: SparseSlotMap<i32> = SparseSlotMap::with_capacity(3);
	///
	/// let k1 = map.insert(1);
	/// let k2 = map.insert(2);
	/// let k3 = map.insert(3);
	///
	/// let v: Vec<_> = map.drain().collect();
	/// assert_eq!(map.len(), 0);
	/// assert_eq!(v, vec![(k1, 1), (k2, 2), (k3, 3)]);
	/// ```
	pub fn drain(&mut self) -> Drain<'_, T, Idx> {
		Drain {
			current: Idx::zero(),
			num_left: self.len(),
			map: self,
		}
	}

	/// Clears the slot map, returning all key-value pairs as an iterator. Keeps
	/// the allocated memory for reuse.
	///
	/// This function must iterate over all slots, empty or not. In the face of
	/// many deleted elements it can be inefficient.
	///
	/// # Examples
	///
	/// ```
	/// use astral::core::collections::SparseSlotMap;
	///
	/// let mut map: SparseSlotMap<i32> = SparseSlotMap::with_capacity(4);
	///
	/// let k1 = map.insert(1);
	/// let k2 = map.insert(2);
	/// let k3 = map.insert(3);
	/// let k4 = map.insert(4);
	///
	/// let evens: Vec<_> = map.drain_filter(|_, val| *val % 2 == 0).collect();
	/// let odds: Vec<_> = map.drain().collect();
	/// assert!(map.is_empty());
	/// assert_eq!(evens, vec![(k2, 2), (k4, 4)]);
	/// assert_eq!(odds, vec![(k1, 1), (k3, 3)]);
	/// ```
	pub fn drain_filter<F>(&mut self, filter: F) -> DrainFilter<'_, T, Idx, F>
	where
		F: FnMut(Key<Idx>, &mut T) -> bool,
	{
		DrainFilter {
			current: Idx::zero(),
			num_left: self.len(),
			map: self,
			pred: filter,
		}
	}
}

impl<T, Idx> Default for SlotMap<T, Idx>
where
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{
	fn default() -> Self {
		Self::new()
	}
}

impl<T, Idx> Debug for SlotMap<T, Idx>
where
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	T: Debug,
	usize: AsPrimitive<Idx>,
{
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		fmt.debug_map().entries(self.iter()).finish()
	}
}

impl<T, Idx> Index<Key<Idx>> for SlotMap<T, Idx>
where
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{
	type Output = T;

	fn index(&self, key: Key<Idx>) -> &Self::Output {
		self.get(key).expect("Invalid key")
	}
}

impl<T, Idx> IndexMut<Key<Idx>> for SlotMap<T, Idx>
where
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{
	fn index_mut(&mut self, key: Key<Idx>) -> &mut Self::Output {
		self.get_mut(key).expect("Invalid key")
	}
}

impl<T, Idx> IntoIterator for SlotMap<T, Idx>
where
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{
	type IntoIter = IntoIter<T, Idx>;
	type Item = (Key<Idx>, T);

	fn into_iter(self) -> Self::IntoIter {
		IntoIter {
			num_left: self.len(),
			slots: self.slots.into_iter().enumerate(),
		}
	}
}

impl<'a, T, Idx> IntoIterator for &'a SlotMap<T, Idx>
where
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{
	type IntoIter = Iter<'a, T, Idx>;
	type Item = (Key<Idx>, &'a T);

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

impl<'a, T, Idx> IntoIterator for &'a mut SlotMap<T, Idx>
where
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{
	type IntoIter = IterMut<'a, T, Idx>;
	type Item = (Key<Idx>, &'a mut T);

	fn into_iter(self) -> Self::IntoIter {
		self.iter_mut()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_create_key() {
		let mut map: SlotMap<u32> = SlotMap::new();
		let key = map.create_key();
		assert!(map.is_empty());
		assert_eq!(map.remove(key), None);
		assert!(map.is_empty());
	}

	#[test]
	fn test_insert_remove() {
		let mut map: SlotMap<u32> = SlotMap::default();
		let a = map.insert(10);
		let b = map.insert(20);
		let c = map.insert(30);
		let d = map.insert(40);
		let e = map.insert(50);
		assert_eq!(map.len(), 5);
		assert_eq!(map.get(a), Some(&10));
		assert_eq!(map.get(c), Some(&30));
		assert_eq!(map.get(e), Some(&50));
		assert!(map.contains_key(a));
		assert!(map.contains_key(b));
		assert!(map.contains_key(c));

		assert_eq!(map.remove(a), Some(10));
		assert_eq!(map.remove(d), Some(40));
		assert_eq!(map.remove(b), Some(20));
		assert!(!map.contains_key(a));
		assert!(!map.contains_key(d));
		assert!(!map.contains_key(b));
		let a = map.insert(100);
		let b = map.insert(200);
		let d = map.insert(400);
		assert!(map.contains_key(a));
		assert!(map.contains_key(d));
		assert!(map.contains_key(b));
		assert_eq!(map.get(a), Some(&100));
		assert_eq!(map.get(b), Some(&200));
		assert_eq!(map.get(d), Some(&400));
	}
}
