// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::{
	fmt::{self, Debug, Formatter},
	mem,
};

use crate::math::num::{NonZero, PrimUnsignedInt};

/// Used to access stored values in a slot map.
///
/// Do not use a key from one slot map in another. The behavior is safe but
/// non-sensical (and might panic in case of out-of-bounds).
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Key<Idx = u32>
where
	Idx: PrimUnsignedInt,
{
	index: Idx,
	version: Idx::NonZero,
}

impl<Idx> Key<Idx>
where
	Idx: PrimUnsignedInt,
{
	pub(super) fn new(index: Idx, version: Idx) -> Self {
		debug_assert!(version != Idx::zero(), "version must not be zero");
		Self {
			index,
			version: unsafe { Idx::NonZero::new_unchecked(version) },
		}
	}

	pub(super) fn index(&self) -> Idx {
		self.index
	}

	pub(super) fn version(&self) -> Idx {
		self.version.get()
	}
}

impl<Idx> Debug for Key<Idx>
where
	Idx: PrimUnsignedInt,
{
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		write!(
			fmt,
			"0x{0:0>idx$x}{1:0>ver$x}",
			self.index,
			self.version.get(),
			idx = mem::size_of::<Idx>() / 2,
			ver = mem::size_of::<Idx>() / 2,
		)
	}
}
