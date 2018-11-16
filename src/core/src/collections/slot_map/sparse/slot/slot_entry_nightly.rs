// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::mem::ManuallyDrop;

use super::PrimUnsignedInt;

pub union SlotEntry<T, Idx>
where
	Idx: PrimUnsignedInt,
{
	value: ManuallyDrop<T>,
	index: Idx,
	reserved: (),
}

impl<T, Idx> SlotEntry<T, Idx>
where
	Idx: PrimUnsignedInt,
{
	pub fn new_from_value(value: T) -> Self {
		Self {
			value: ManuallyDrop::new(value),
		}
	}
	pub fn new_from_index(index: Idx) -> Self {
		Self { index }
	}
	pub fn new_reserved() -> Self {
		Self { reserved: () }
	}

	pub unsafe fn value(&self) -> &T {
		&self.value
	}

	pub unsafe fn value_mut(&mut self) -> &mut T {
		&mut self.value
	}

	pub unsafe fn index(&self) -> Idx {
		self.index
	}

	pub unsafe fn into_inner(self) -> T {
		ManuallyDrop::into_inner(self.value)
	}

	pub unsafe fn drop(&mut self) {
		ManuallyDrop::drop(&mut self.value)
	}
}
