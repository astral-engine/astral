// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::{
	iter::{Enumerate, ExactSizeIterator, FusedIterator},
	slice::Iter as SliceIter,
};

use crate::math::num::{AsPrimitive, PrimUnsignedInt};

use crate::collections::slot_map::{sparse::Slot, Key};

// TODO(#10): Use elided lifetimes
#[derive(Debug)]
pub struct Iter<'a, T, Idx>
where
	T: 'a,
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{
	pub(super) num_left: Idx,
	pub(super) slots: Enumerate<SliceIter<'a, Slot<T, Idx>>>,
}

impl<'a, T, Idx> Iterator for Iter<'a, T, Idx>
where
	T: 'a,
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{
	type Item = (Key<Idx>, &'a T);

	fn next(&mut self) -> Option<Self::Item> {
		while let Some((idx, slot)) = self.slots.next() {
			if slot.occupied() {
				let key = Key::new(idx.as_(), slot.version());
				self.num_left -= Idx::one();
				return Some((key, &slot.value()));
			}
		}

		None
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(self.num_left.as_(), Some(self.num_left.as_()))
	}
}

impl<'a, T, Idx> FusedIterator for Iter<'a, T, Idx>
where
	T: 'a,
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{}

impl<'a, T, Idx> ExactSizeIterator for Iter<'a, T, Idx>
where
	T: 'a,
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{}
