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

#![allow(clippy::module_name_repetitions)]

use std::{
	error,
	fmt::{self, Debug, Display, Formatter},
	str,
	string,
};

/// Errors which can occur when attempting to interpret a sequence of [`u8`] as a string.
///
/// As such, the `from_utf8` family of functions and methods for both [`Name`]s and [`Text`]s make
/// use of this error, for example.
///
/// [`Name`]: struct.Name.html
/// [`Text`]: struct.Text.html
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Utf8Error {
	inner: str::Utf8Error,
}

impl Utf8Error {
	pub(super) fn from_std(std: str::Utf8Error) -> Self {
		Self { inner: std }
	}

	/// Returns the index in the given string up to which valid UTF-8 was verified.
	///
	/// It is the maximum index such that `from_utf8(&input[..index])` would return `Ok(_)`.
	///
	/// # Example
	///
	/// ```
	/// # use astral::thirdparty::slog;
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
	/// use astral::string::Name;
	///
	/// // some invalid bytes, in a vector
	/// let sparkle_heart = &[0, 159, 146, 150];
	///
	/// // Name::from_utf8 returns a Utf8Error
	/// let error = Name::from_utf8(sparkle_heart, &string_subsystem).unwrap_err();
	///
	/// // the second byte is invalid here
	/// assert_eq!(1, error.valid_up_to());
	/// ```
	#[inline]
	pub fn valid_up_to(&self) -> usize {
		self.inner.valid_up_to()
	}

	/// Provide more information about the failure:
	///
	/// * `None`: the end of the input was reached unexpectedly. `self.valid_up_to()` is 1 to 3
	///   bytes from the end of the input. If a byte stream (such as a file or a network socket) is
	///   being decoded incrementally, this could be a valid `char` whose UTF-8 byte sequence is
	///   spanning multiple chunks.
	///
	/// * `Some(len)`: an unexpected byte was encountered. The length provided is that of the
	///   invalid byte sequence that starts at the index given by `valid_up_to()`. Decoding should
	///   resume after that sequence (after inserting a [`U+FFFD REPLACEMENT CHARACTER`][U+FFFD]) in
	///   case of lossy decoding.
	///
	/// [U+FFFD]: std::char::REPLACEMENT_CHARACTER
	#[inline]
	pub fn error_len(&self) -> Option<usize> {
		self.inner.error_len()
	}
}

impl Debug for Utf8Error {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		Debug::fmt(&self.inner, fmt)
	}
}

impl Display for Utf8Error {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.inner, fmt)
	}
}

impl error::Error for Utf8Error {}

/// A possible error value when converting a [`Name`] or [`Text`] from an UTF-16 byte slice.
///
/// This type is the error type for the `from_utf16` method on [`Name`] or [`Text`].
///
/// [`Name`]: struct.Name.html
/// [`Text`]: struct.Text.html
///
/// # Example
///
/// ```
/// # use astral::thirdparty::slog;
///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
///	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
/// use astral::string::Name;
///
/// // 𝄞mu<invalid>ic
/// let v = &[0xD834, 0xDD1E, 0x006d, 0x0075,
///           0xD800, 0x0069, 0x0063];
///
/// assert!(Name::from_utf16(v, &string_subsystem).is_err());
/// ```
pub struct Utf16Error {
	pub(super) inner: string::FromUtf16Error,
}

impl Utf16Error {
	pub(super) fn from_std(std: string::FromUtf16Error) -> Self {
		Self { inner: std }
	}
}

impl Debug for Utf16Error {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		Debug::fmt(&self.inner, fmt)
	}
}

impl Display for Utf16Error {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.inner, fmt)
	}
}

impl error::Error for Utf16Error {}
