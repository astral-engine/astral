// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
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
/// # use astral::{third_party::slog, Engine, core::{System, string}};
///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
///	# let engine = Engine::new(&logger);
///	# let system = System::new(&engine);
///	# let string_subsystem = string::Subsystem::new(64, &system);
/// use astral::core::string::StringId;
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
