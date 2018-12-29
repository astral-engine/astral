// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::{
	iter::{Enumerate, ExactSizeIterator, FusedIterator},
	vec,
};

use crate::math::num::{AsPrimitive, PrimUnsignedInt};

use crate::collections::slot_map::{sparse::Slot, Key};

#[derive(Debug)]
pub struct IntoIter<T, Idx>
where
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{
	pub(super) num_left: Idx,
	pub(super) slots: Enumerate<vec::IntoIter<Slot<T, Idx>>>,
}

impl<T, Idx> Iterator for IntoIter<T, Idx>
where
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{
	type Item = (Key<Idx>, T);

	fn next(&mut self) -> Option<Self::Item> {
		while let Some((idx, mut slot)) = self.slots.next() {
			if slot.occupied() {
				let key = Key::new(idx.as_(), slot.version());
				self.num_left -= Idx::one();
				return Some((key, slot.take()));
			}
		}

		None
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		(self.num_left.as_(), Some(self.num_left.as_()))
	}
}

impl<T, Idx> FusedIterator for IntoIter<T, Idx>
where
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{
}

impl<T, Idx> ExactSizeIterator for IntoIter<T, Idx>
where
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{
}
