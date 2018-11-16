// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::{
	fmt::{Binary, Debug, Display, LowerHex, Octal, UpperHex},
	num::{
		NonZeroU128, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8,
		NonZeroUsize,
	},
	ops::{AddAssign, DivAssign, MulAssign, RemAssign, SubAssign},
};

use super::{
	NonZero, PrimInt, Unsigned, WrappingAdd, WrappingMul, WrappingShl,
	WrappingShr, WrappingSub,
};

/// Functions for primitive unsigned integral types.
pub trait PrimUnsignedInt:
	Unsigned
	+ PrimInt
	+ WrappingAdd
	+ WrappingSub
	+ WrappingMul
	+ WrappingShl
	+ WrappingShr
	+ AddAssign
	+ for<'a> AddAssign<&'a Self>
	+ SubAssign
	+ for<'a> SubAssign<&'a Self>
	+ MulAssign
	+ for<'a> MulAssign<&'a Self>
	+ DivAssign
	+ for<'a> DivAssign<&'a Self>
	+ RemAssign
	+ for<'a> RemAssign<&'a Self>
	+ Debug
	+ Display
	+ Binary
	+ LowerHex
	+ UpperHex
	+ Octal
{
	/// The [`NonZero`] part for this type.
	///
	/// [`NonZero`]: trait.NonZero.html
	type NonZero: NonZero<Int = Self>;
}

macro_rules! prim_unsigned_int_traits {
    ( $( $Ty: ident($Int: ty); )+ ) => {
        $(
            impl PrimUnsignedInt for $Int {
                type NonZero = $Ty;
            }
        )+
    };
}

prim_unsigned_int_traits! {
	NonZeroU8(u8);
	NonZeroU16(u16);
	NonZeroU32(u32);
	NonZeroU64(u64);
	NonZeroU128(u128);
	NonZeroUsize(usize);
}
