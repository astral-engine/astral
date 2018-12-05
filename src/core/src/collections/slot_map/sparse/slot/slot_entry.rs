// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::mem::ManuallyDrop;

use super::PrimUnsignedInt;

// TODO: Use untagged_unions.
pub enum SlotEntry<T, Idx>
where
	Idx: PrimUnsignedInt,
{
	Value(ManuallyDrop<T>),
	Index(Idx),
	Reserved,
}

impl<T, Idx> SlotEntry<T, Idx>
where
	Idx: PrimUnsignedInt,
{
	pub fn new_from_value(value: T) -> Self {
		SlotEntry::Value(ManuallyDrop::new(value))
	}
	pub fn new_from_index(index: Idx) -> Self {
		SlotEntry::Index(index)
	}
	pub fn new_reserved() -> Self {
		SlotEntry::Reserved
	}

	pub unsafe fn value(&self) -> &T {
		if let SlotEntry::Value(value) = self {
			value
		} else {
			panic!("Expected value")
		}
	}

	pub unsafe fn value_mut(&mut self) -> &mut T {
		if let SlotEntry::Value(value) = self {
			value
		} else {
			panic!("Expected value")
		}
	}

	pub unsafe fn index(&self) -> Idx {
		if let SlotEntry::Index(index) = self {
			*index
		} else {
			panic!("Expected index")
		}
	}

	pub unsafe fn into_inner(self) -> T {
		if let SlotEntry::Value(value) = self {
			ManuallyDrop::into_inner(value)
		} else {
			panic!("Expected value")
		}
	}

	pub unsafe fn drop(&mut self) {
		if let SlotEntry::Value(value) = self {
			ManuallyDrop::drop(value)
		} else {
			panic!("Expected value")
		}
	}
}
