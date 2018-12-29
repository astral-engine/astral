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

use std::{hash::Hasher, u32};

/// An implementation of the [Murmur3 Hash].
///
/// [Murmur3 Hash]: https://en.wikipedia.org/wiki/MurmurHash#MurmurHash3
///
/// # Panics
///
/// Panics if values with a size greater than 8 bytes are passed in.
///
/// # Example
///
/// ```
/// use std::hash::{Hash, Hasher};
///
/// use astral::util::hash::Murmur3;
///
/// let mut hasher = Murmur3::default();
/// Hash::hash_slice("Hello World!".as_bytes(), &mut hasher);
/// assert_eq!(hasher.finish(), 3691591037);
/// ```
#[derive(Debug, Clone, Default)]
#[allow(missing_copy_implementations)]
pub struct Murmur3 {
	seed: u32,
}

impl Murmur3 {
	const C1: u32 = 0xCC9E_2D51;
	const C2: u32 = 0x1B87_3593;
	const M: u32 = 5;
	const N: u32 = 0xE654_6B64;
	const R1: u32 = 15;
	const R2: u32 = 13;

	fn write_chunk(&mut self, chunk: [u8; 4]) {
		// ToDo(#4): Use u32::from_ne_bytes
		#[cfg(not(unstable))]
		let mut k = unsafe { std::mem::transmute::<_, u32>(chunk) }.to_le();
		#[cfg(unstable)]
		let mut k = u32::from_ne_bytes(chunk).to_le();

		k = u32::wrapping_mul(k, Self::C1);
		k = u32::rotate_left(k, Self::R1);
		k = u32::wrapping_mul(k, Self::C2);

		self.seed ^= k;
	}
}

impl Hasher for Murmur3 {
	fn finish(&self) -> u64 {
		self.seed.into()
	}

	#[allow(clippy::cast_possible_truncation)]
	fn write(&mut self, bytes: &[u8]) {
		for chunk in bytes.chunks(4) {
			match chunk.len() {
				1 => self.write_chunk([chunk[0], 0, 0, 0]),
				2 => self.write_chunk([chunk[0], chunk[1], 0, 0]),
				3 => self.write_chunk([chunk[0], chunk[1], chunk[2], 0]),
				4 => {
					self.write_chunk([chunk[0], chunk[1], chunk[2], chunk[3]]);
					self.seed = u32::rotate_left(self.seed, Self::R2);
					self.seed = u32::wrapping_mul(self.seed, Self::M);
					self.seed = u32::wrapping_add(self.seed, Self::N);
				}
				_ => unreachable!("chunk size is not 4"),
			}
		}

		self.seed ^= bytes.len() as u32;

		self.seed ^= self.seed >> 16;
		self.seed = u32::wrapping_mul(self.seed, 0x85EB_CA6B);
		self.seed ^= self.seed >> 13;
		self.seed = u32::wrapping_mul(self.seed, 0xC2B2_AE35);
		self.seed ^= self.seed >> 16;
	}
}
