// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::iter::{ExactSizeIterator, FusedIterator};

use crate::math::num::{AsPrimitive, PrimUnsignedInt};

use super::Iter;

#[derive(Debug)]
pub struct Values<'a, T, Idx>(pub(super) Iter<'a, T, Idx>)
where
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>;

impl<'a, T, Idx> Iterator for Values<'a, T, Idx>
where
	T: 'a,
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|(_, v)| v)
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.0.size_hint()
	}
}

impl<'a, T, Idx> FusedIterator for Values<'a, T, Idx>
where
	T: 'a,
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{
}

impl<'a, T, Idx> ExactSizeIterator for Values<'a, T, Idx>
where
	T: 'a,
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{
}