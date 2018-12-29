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
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, December 2018

use std::num::NonZeroU32;

use super::Subsystem;

/// An opaque struct for fast comparison between strings.
///
/// `StringId` represents a unique identifier to a string.
///
/// # Example
///
/// ```
///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
///	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
/// use astral::string::StringId;
///
/// let id1 = StringId::new("foo", &string_subsystem);
/// let id2 = StringId::new("bar", &string_subsystem);
/// let id3 = StringId::new("foo", &string_subsystem);
///
/// assert_ne!(id1, id2);
/// assert_eq!(id1, id3);
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct StringId(NonZeroU32);

impl StringId {
	pub(super) fn from_raw_parts(id: u32) -> Self {
		StringId(NonZeroU32::new(id + 1).expect("string id overflow"))
	}

	/// Construcs a new `StringId` from the given string in the specified [`Subsystem`].
	///
	/// If the string was used before, the `StringId` will be equal.
	///
	/// # Example
	pub fn new<S>(string: S, subsystem: &Subsystem) -> Self
	where
		S: AsRef<str>,
	{
		subsystem.create_string_id(string)
	}

	pub(crate) fn get(self) -> u32 {
		self.0.get() - 1
	}
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_size() {
		assert_eq!(std::mem::size_of::<StringId>(), 4);
	}
}
