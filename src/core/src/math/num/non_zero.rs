// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::num::{
	NonZeroU128, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
};

use super::PrimUnsignedInt;

/// Functions for primitive type, which has a non-zero correspondant.
pub trait NonZero: Copy + Sized {
	type Int: PrimUnsignedInt<NonZero = Self>;

	/// Create a non-zero without checking the value.
	///
	/// # Safety
	///
	/// The value must not be zero.
	unsafe fn new_unchecked(n: Self::Int) -> Self;

	/// Create a non-zero if the given value is not zero.
	fn new(n: Self::Int) -> Option<Self>;

	/// Returns the value as the primitive type.
	fn get(self) -> Self::Int;
}

macro_rules! nonzero_traits {
    ( $( $Ty: ident($Int: ty); )+ ) => {
        $(
            impl NonZero for $Ty {
                type Int = $Int;

                unsafe fn new_unchecked(n: Self::Int) -> Self {
                    Self::new_unchecked(n)
                }
                // TODO(#7): Use tool-lints
                #[cfg_attr(feature = "cargo-clippy", allow(new_ret_no_self))]
                fn new(n: Self::Int) -> Option<Self> {
                    Self::new(n)
                }
                fn get(self) -> Self::Int {
                    self.get()
                }
            }
        )+
    };
}

nonzero_traits! {
	NonZeroU8(u8);
	NonZeroU16(u16);
	NonZeroU32(u32);
	NonZeroU64(u64);
	NonZeroU128(u128);
	NonZeroUsize(usize);
}
