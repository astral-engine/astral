// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::iter::{ExactSizeIterator, FusedIterator};

use crate::math::num::{AsPrimitive, PrimUnsignedInt};

use super::{Key, SlotMap};

// TODO(#10): Use elided lifetimes
#[derive(Debug)]
pub struct Drain<'a, T, Idx>
where
	T: 'a,
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{
	pub(super) num_left: Idx,
	pub(super) map: &'a mut SlotMap<T, Idx>,
	pub(super) current: Idx,
}

impl<'a, T, Idx> Iterator for Drain<'a, T, Idx>
where
	T: 'a,
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
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
				let slot = self.map.slots.get_unchecked(idx.as_());
				key = Key::new(idx, slot.version());
				if slot.occupied() {
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
		(self.num_left.as_(), Some(self.num_left.as_()))
	}
}

impl<'a, T, Idx> FusedIterator for Drain<'a, T, Idx>
where
	T: 'a,
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{}

impl<'a, T, Idx> ExactSizeIterator for Drain<'a, T, Idx>
where
	T: 'a,
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{}

impl<'a, T, Idx> Drop for Drain<'a, T, Idx>
where
	T: 'a,
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{
	fn drop(&mut self) {
		self.for_each(|_drop| {});
	}
}
