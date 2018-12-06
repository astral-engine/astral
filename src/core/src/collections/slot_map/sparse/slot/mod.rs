// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

#[cfg(not(unstable))]
mod slot_entry;
#[cfg(unstable)]
mod slot_entry_nightly;

use std::{
	fmt::{self, Debug, Formatter},
	mem,
};

use crate::math::num::PrimUnsignedInt;

#[cfg(not(unstable))]
use self::slot_entry::SlotEntry;
#[cfg(unstable)]
use self::slot_entry_nightly::SlotEntry;

pub struct Slot<T, Idx>
where
	Idx: PrimUnsignedInt,
{
	entry: SlotEntry<T, Idx>,
	version: Idx,
}

impl<T, Idx> Slot<T, Idx>
where
	Idx: PrimUnsignedInt,
{
	fn occupied_bit() -> Idx {
		Idx::one() << (mem::size_of::<Idx>() * 8 - 1)
	}

	fn reserved_bit() -> Idx {
		Idx::one() << (mem::size_of::<Idx>() * 8 - 2)
	}

	pub fn max_version() -> Idx {
		Idx::max_value() & !Self::occupied_bit() & !Self::reserved_bit()
	}

	pub fn version(&self) -> Idx {
		self.version & !Self::occupied_bit() & !Self::reserved_bit()
	}

	pub fn occupied(&self) -> bool {
		let occupied = self.version & Self::occupied_bit() == Self::occupied_bit();
		if occupied {
			debug_assert!(self.version & Self::reserved_bit() == Idx::zero());
		}
		occupied
	}

	pub fn reserved(&self) -> bool {
		let reserved = self.version & Self::reserved_bit() == Self::reserved_bit();
		if reserved {
			debug_assert!(self.version & Self::occupied_bit() == Idx::zero());
		}
		reserved
	}

	pub fn free(&self) -> bool {
		!self.occupied() && !self.reserved()
	}

	pub fn new() -> Self {
		Self {
			entry: SlotEntry::new_reserved(),
			version: Idx::one() | Self::reserved_bit(),
		}
	}

	pub fn index(&mut self) -> Idx {
		debug_assert!(self.free());
		unsafe { self.entry.index() }
	}

	pub fn set_index(&mut self, index: Idx) -> Option<T> {
		let entry = mem::replace(&mut self.entry, SlotEntry::new_from_index(index));
		let occupied = self.occupied();
		self.version = self.version();
		debug_assert!(self.free());
		if occupied {
			unsafe { Some(entry.into_inner()) }
		} else {
			None
		}
	}

	pub fn value(&self) -> &T {
		debug_assert!(self.occupied());
		unsafe { self.entry.value() }
	}

	pub fn value_mut(&mut self) -> &mut T {
		debug_assert!(self.occupied());
		unsafe { self.entry.value_mut() }
	}

	pub fn set_value(&mut self, value: T) -> Option<T> {
		let entry = mem::replace(&mut self.entry, SlotEntry::new_from_value(value));
		let occupied = self.occupied();
		self.version = self.version() | Self::occupied_bit();
		debug_assert!(self.occupied());
		if occupied {
			unsafe { Some(entry.into_inner()) }
		} else {
			None
		}
	}

	pub fn next_version(&self) -> Idx {
		if self.version == Self::max_version() {
			Idx::one()
		} else {
			self.version + Idx::one()
		}
	}

	pub fn increment_version(&mut self) {
		self.version = self.next_version();
	}

	pub fn take(&mut self) -> T {
		debug_assert!(self.occupied());
		unsafe { mem::replace(&mut self.entry, SlotEntry::new_reserved()).into_inner() }
	}
}

impl<T, Idx> Clone for Slot<T, Idx>
where
	T: Clone,
	Idx: PrimUnsignedInt,
{
	fn clone(&self) -> Self {
		Self {
			entry: unsafe {
				if self.occupied() {
					SlotEntry::new_from_value(self.entry.value().clone())
				} else if self.reserved() {
					SlotEntry::new_reserved()
				} else {
					SlotEntry::new_from_index(self.entry.index())
				}
			},
			version: self.version,
		}
	}
}

impl<T, Idx> Debug for Slot<T, Idx>
where
	T: Debug,
	Idx: Debug + PrimUnsignedInt,
	Idx: Debug + PrimUnsignedInt,
{
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		let mut debug = fmt.debug_struct("Slot");
		unsafe {
			if self.occupied() {
				debug.field("value", self.entry.value());
			} else if !self.reserved() {
				debug.field("next_free", &self.entry.index());
			}
		}
		debug.field("version", &self.version());
		debug.finish()
	}
}

impl<T, Idx> Drop for Slot<T, Idx>
where
	Idx: PrimUnsignedInt,
{
	fn drop(&mut self) {
		if mem::needs_drop::<T>() && self.occupied() {
			unsafe {
				self.entry.drop();
			}
		}
	}
}
