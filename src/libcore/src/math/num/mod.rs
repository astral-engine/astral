// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

//! Additional functionality for numerics.
//!
//! This module provides traits that are useful when doing numerical work.
//! See the individual documentation for each piece for more information.

pub use num_traits::{
	AsPrimitive, Bounded, Num, NumCast, NumOps, One, PrimInt, Saturating, Signed, Unsigned,
	WrappingAdd, WrappingMul, WrappingShl, WrappingShr, WrappingSub, Zero,
};

mod non_zero;
pub use self::non_zero::NonZero;

mod prim_unsigned_int;
pub use self::prim_unsigned_int::PrimUnsignedInt;
