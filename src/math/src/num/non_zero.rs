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

use std::num::{NonZeroU128, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize};

use super::PrimUnsignedInt;

/// Functions for primitive type, which has a non-zero correspondant.
pub trait NonZero: Copy + Sized {
	/// The primitive unsigned int correspondant.
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
	#[allow(clippy::new_ret_no_self)]
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
