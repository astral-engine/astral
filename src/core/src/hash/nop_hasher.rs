// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::{hash::Hasher, ptr};

/// An implementation of [`Hasher`] hasher which only accepts values with a size
/// of 8 bytes or an integral value fitting into 8 bytes.
///
/// [`Hasher`]: https://doc.rust-lang.org/std/hash/trait.Hasher.html
///
/// # Panics
///
/// Panics if values with a size greater than 8 bytes are passed in.
///
/// # Examples
///
/// Usage:
///
/// ```
/// # extern crate astral;
/// use std::hash::{Hash, Hasher};
/// use astral::core::hash::NopHasher;
///
/// let mut hasher = NopHasher::default();
/// 1234_5678_u32.hash(&mut hasher);
/// assert_eq!(hasher.finish(), 1234_5678);
/// ```
///
/// Slices and arrays cannot be hashed directly, since their len is also hashed.
/// `Hash::hash_slice` may be used instead:
///
/// ```
/// # extern crate astral;
/// use std::hash::{Hash, Hasher};
/// use astral::core::hash::NopHasher;
///
/// let mut hasher = NopHasher::default();
/// let arr = [0x12_u8, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
/// Hash::hash_slice(&arr, &mut hasher);
/// assert_eq!(hasher.finish(), 0x1234_5678_9ABC_DEF0_u64.to_be());
/// ```
#[derive(Debug, Clone, Default)]
pub struct NopHasher {
	value: u64,
}

impl Hasher for NopHasher {
	fn finish(&self) -> u64 {
		self.value
	}

	fn write(&mut self, bytes: &[u8]) {
		debug_assert!(
			bytes.len() == 8,
			"Only values with a size of 8 bytes or integrals that fit into 8 bytes are allowed."
		);
		unsafe {
			// TODO(#7): Use tool-lints
			#[cfg_attr(feature = "cargo-clippy", allow(cast_ptr_alignment))]
			ptr::copy_nonoverlapping(
				bytes.as_ptr() as *const u64,
				&mut self.value,
				1,
			);
		}
	}

	fn write_u8(&mut self, i: u8) {
		self.write_u64(i.into())
	}

	fn write_u16(&mut self, i: u16) {
		self.write_u64(i.into())
	}

	fn write_u32(&mut self, i: u32) {
		self.write_u64(i.into())
	}

	fn write_usize(&mut self, i: usize) {
		self.write_u64(i as u64)
	}

	fn write_i8(&mut self, i: i8) {
		self.write_i64(i.into())
	}

	fn write_i16(&mut self, i: i16) {
		self.write_i64(i.into())
	}

	fn write_i32(&mut self, i: i32) {
		self.write_i64(i.into())
	}

	fn write_isize(&mut self, i: isize) {
		self.write_i64(i as i64)
	}
}

#[cfg(test)]
mod tests {
	use super::NopHasher;
	use std::hash::{Hash, Hasher};

	fn hash<T: Hash>(t: T) -> u64 {
		let mut hasher = NopHasher::default();
		t.hash(&mut hasher);
		hasher.finish()
	}

	// TODO(#7): Use tool-lints
	#[cfg_attr(feature = "cargo-clippy", allow(cast_sign_loss))]
	#[test]
	fn test_integrals() {
		assert_eq!(hash(10_u8), 10);
		assert_eq!(hash(10_u16), 10);
		assert_eq!(hash(10_u32), 10);
		assert_eq!(hash(10_u64), 10);
		assert_eq!(hash(10_usize), 10);

		assert_eq!(hash(10_i8), 10);
		assert_eq!(hash(10_i16), 10);
		assert_eq!(hash(10_i32), 10);
		assert_eq!(hash(10_i64), 10);
		assert_eq!(hash(10_isize), 10);

		assert_eq!(hash(-10_i8), -10_i8 as u64);
		assert_eq!(hash(-10_i16), -10_i16 as u64);
		assert_eq!(hash(-10_i32), -10_i32 as u64);
		assert_eq!(hash(-10_i64), -10_i64 as u64);
		assert_eq!(hash(-10_isize), -10_isize as u64);
	}

	#[test]
	fn test_array() {
		let mut hasher = NopHasher::default();
		let le_array = [0xF0_u8, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12];
		Hash::hash_slice(&le_array, &mut hasher);
		assert_eq!(hasher.finish(), 0x1234_5678_9ABC_DEF0_u64.to_le());

		let mut hasher = NopHasher::default();
		let be_array = [0x12_u8, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0];
		Hash::hash_slice(&be_array, &mut hasher);
		assert_eq!(hasher.finish(), 0x1234_5678_9ABC_DEF0_u64.to_be());
	}
}
