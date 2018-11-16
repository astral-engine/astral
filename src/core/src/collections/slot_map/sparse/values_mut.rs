// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::iter::{ExactSizeIterator, FusedIterator};

use crate::math::num::{AsPrimitive, PrimUnsignedInt};

use super::IterMut;

// TODO(#10): Use elided lifetimes
#[derive(Debug)]
pub struct ValuesMut<'a, T, Idx>(pub(super) IterMut<'a, T, Idx>)
where
	T: 'a,
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>;

impl<'a, T, Idx> Iterator for ValuesMut<'a, T, Idx>
where
	T: 'a,
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{
	type Item = &'a mut T;

	fn next(&mut self) -> Option<Self::Item> {
		self.0.next().map(|(_, v)| v)
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		self.0.size_hint()
	}
}

impl<'a, T, Idx> FusedIterator for ValuesMut<'a, T, Idx>
where
	T: 'a,
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{}

impl<'a, T, Idx> ExactSizeIterator for ValuesMut<'a, T, Idx>
where
	T: 'a,
	Idx: PrimUnsignedInt + AsPrimitive<usize>,

	usize: AsPrimitive<Idx>,
{}
