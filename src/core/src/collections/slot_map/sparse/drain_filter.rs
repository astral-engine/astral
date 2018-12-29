// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::{
	fmt::{self, Debug, Formatter},
	iter::FusedIterator,
};

use crate::math::num::{AsPrimitive, PrimUnsignedInt};

use super::{Key, SlotMap};

pub struct DrainFilter<'a, T, Idx, F>
where
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
	F: FnMut(Key<Idx>, &mut T) -> bool,
{
	pub(super) num_left: Idx,
	pub(super) map: &'a mut SlotMap<T, Idx>,
	pub(super) current: Idx,
	pub(super) pred: F,
}

impl<T, Idx, F> Debug for DrainFilter<'_, T, Idx, F>
where
	T: Debug,
	Idx: PrimUnsignedInt + AsPrimitive<usize>,
	F: FnMut(Key<Idx>, &mut T) -> bool,

	usize: AsPrimitive<Idx>,
{
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		fmt.debug_struct("DrainFilter")
			.field("map", self.map)
			.field("current", &self.current)
			.field("size_hint", &self.size_hint())
			.finish()
	}
}

impl<'a, T, Idx, F> Iterator for DrainFilter<'a, T, Idx, F>
where
	T: 'a,
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
	F: FnMut(Key<Idx>, &mut T) -> bool,
{
	type Item = (Key<Idx>, T);

	fn next(&mut self) -> Option<Self::Item> {
		let len = self.map.slots.len().as_();
		while self.current < len {
			let idx = self.current;
			self.current += Idx::one();

			unsafe {
				let slot = self.map.slots.get_unchecked_mut(idx.as_());
				let key = Key::new(idx, slot.version());
				if slot.occupied() && (self.pred)(key, slot.value_mut()) {
					self.num_left -= Idx::one();
					return Some((key, self.map.remove(key).unwrap()));
				}
			}
		}

		None
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(0, Some(self.num_left.as_()))
	}
}

impl<'a, T, Idx, F> FusedIterator for DrainFilter<'a, T, Idx, F>
where
	T: 'a,
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
	F: FnMut(Key<Idx>, &mut T) -> bool,
{
}

impl<'a, T, Idx, F> Drop for DrainFilter<'a, T, Idx, F>
where
	T: 'a,
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
	F: FnMut(Key<Idx>, &mut T) -> bool,
{
	fn drop(&mut self) {
		self.for_each(|_drop| {});
	}
}
