// Copyright (c) Astral Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::{
	fmt::{Binary, Debug, Display, LowerHex, Octal, UpperHex},
	num::{NonZeroU128, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize},
	ops::{AddAssign, DivAssign, MulAssign, RemAssign, SubAssign},
};

use super::{
	NonZero,
	PrimInt,
	Unsigned,
	WrappingAdd,
	WrappingMul,
	WrappingShl,
	WrappingShr,
	WrappingSub,
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
