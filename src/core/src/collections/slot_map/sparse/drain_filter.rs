// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::iter::FusedIterator;

use crate::math::num::{AsPrimitive, PrimUnsignedInt};

use super::{Key, SlotMap};

// TODO(#10): Use elided lifetimes
#[derive(Debug)]
pub struct DrainFilter<'a, T, Idx, F>
where
	T: 'a,
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
	F: FnMut(Key<Idx>, &mut T) -> bool,
{
	pub(super) num_left: Idx,
	pub(super) map: &'a mut SlotMap<T, Idx>,
	pub(super) current: Idx,
	pub(super) pred: F,
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
			// TODO(#6): Use NLL
			let mut remove = false;
			let key;
			unsafe {
				let slot = self.map.slots.get_unchecked_mut(idx.as_());
				key = Key::new(idx, slot.version());
				if slot.occupied() && (self.pred)(key, slot.value_mut()) {
					remove = true;
				}
			}
			if remove {
				self.num_left -= Idx::one();
				return Some((key, self.map.remove(key).unwrap()));
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
{}

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
