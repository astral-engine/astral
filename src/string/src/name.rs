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
	borrow::Cow,
	cmp::{Ordering, PartialEq, PartialOrd},
	error::Error,
	ffi::OsString,
	fmt::{self, Debug, Display, Formatter},
	hash::{BuildHasher, BuildHasherDefault, Hash, Hasher},
	num::NonZeroU32,
	path::PathBuf,
	str::{self, FromStr},
};

use astral_util::hash::Murmur3;

use super::{StringId, Subsystem, Text, Utf16Error, Utf8Error};

/// A UTF-8 encoded, immutable string optimized for numeric suffixes.
///
/// # Example
///
/// `Name` can be created from a literal string:
///
/// ```
/// # use astral_thirdparty::slog;
///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
///	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
/// use astral::string::Name;
///
/// let name = Name::new("foo", &string_subsystem);
/// assert_eq!(name, "foo");
/// ```
///
/// # Representation
///
/// `Name` stores a [`StringId`], a reference to a [`Subsystem`], and an optional numeric suffix. When
/// a new `Name` is created, it is first checked if the string already exists. If so, it gets the
/// same index as the existing one. If not, a new entry is created.
///
/// The suffix is only used for reusing the same string multiple times when the string only differs
/// at a numeric suffix. A suffix with leading zeros cannot be optimized!
///
/// [`StringId`]: struct.StringId.html
/// [`Subsystem`]: struct.Subsystem.html
pub struct Name<'system, H = BuildHasherDefault<Murmur3>> {
	id: StringId,
	number: Option<NonZeroU32>,
	system: &'system Subsystem<H>,
}

impl<'system, H> Name<'system, H>
where
	H: BuildHasher,
{
	/// Creates a `Text` from the given string literal in the specified [`Subsystem`].
	///
	/// [`Subsystem`]: struct.Subsystem.html
	///
	/// # Example
	///
	/// ```
	/// # use astral_thirdparty::slog;
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
	/// use astral::string::Name;
	///
	/// let name = Name::new("foo", &string_subsystem);
	/// assert_eq!(name, name);
	/// ```
	pub fn new<T>(string: T, system: &'system Subsystem<H>) -> Self
	where
		T: AsRef<str>,
	{
		let (string, number) = Self::split_string(string.as_ref());
		let id = system.create_string_id(string);
		unsafe { Self::from_raw_parts(id, number, system) }
	}

	/// Converts a slice of bytes to a `Name`.
	///
	/// `Name` requires that it is valid UTF-8. `from_utf8` checks to ensure
	/// that the bytes are valid UTF-8, and then does the conversion.
	///
	/// If you are sure that the byte slice is valid UTF-8, and you don't want to
	/// incur the overhead of the validity check, there is an unsafe version of
	/// this function, [`from_utf8_unchecked`], which has the same
	/// behavior but skips the check.
	///
	/// [`from_utf8_unchecked`]: #method.from_utf8_unchecked
	///
	/// # Errors
	///
	/// Returns [`Err`] if the slice is not UTF-8 with a description as to why the
	/// provided slice is not UTF-8.
	///
	/// See the docs for [`Utf8Error`] for more details on the kinds of
	/// errors that can be returned.
	///
	/// [`Utf8Error`]: struct.Utf8Error.html
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # use astral_thirdparty::slog;
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
	/// use astral::string::Name;
	///
	/// // some bytes, in a vector
	/// let sparkle_heart = &[240, 159, 146, 150];
	///
	/// // We know these bytes are valid, so just use `unwrap()`.
	/// let sparkle_heart = Name::from_utf8(sparkle_heart, &string_subsystem).unwrap();
	///
	/// assert_eq!("💖", sparkle_heart);
	/// ```
	///
	/// Incorrect bytes:
	///
	/// ```
	/// # use astral_thirdparty::slog;
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
	/// use astral::string::Name;
	///
	/// // some invalid bytes, in a vector
	/// let sparkle_heart = &[0, 159, 146, 150];
	///
	/// assert!(Name::from_utf8(sparkle_heart, &string_subsystem).is_err());
	/// ```
	pub fn from_utf8(v: &[u8], system: &'system Subsystem<H>) -> Result<Self, Utf8Error> {
		Ok(Self::new(
			str::from_utf8(v).map_err(Utf8Error::from_std)?,
			system,
		))
	}

	/// Converts a slice of bytes to a `Name`, including invalid characters.
	///
	/// `Name` requires that it is valid UTF-8. [`from_utf8`] checks to ensure
	/// that the bytes are valid UTF-8. During this conversion,
	/// `from_utf8_lossy` will replace any invalid UTF-8 sequences with
	/// [`U+FFFD REPLACEMENT CHARACTER`][U+FFFD], which looks like this: �
	///
	/// If you are sure that the byte slice is valid UTF-8, and you don't want
	/// to incur the overhead of the conversion, there is an unsafe version
	/// of this function, [`from_utf8_unchecked`], which has the same behavior
	/// but skips the checks.
	///
	/// [U+FFFD]: std::char::REPLACEMENT_CHARACTER
	/// [`from_utf8_unchecked`]: #method.from_utf8_unchecked
	/// [`from_utf8`]: #method.from_utf8
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # use astral_thirdparty::slog;
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
	/// use astral::string::Name;
	///
	/// // some bytes, in a vector
	/// let sparkle_heart = vec![240, 159, 146, 150];
	///
	/// let sparkle_heart = Name::from_utf8_lossy(&sparkle_heart, &string_subsystem);
	///
	/// assert_eq!("💖", sparkle_heart);
	/// ```
	///
	/// Incorrect bytes:
	///
	/// ```
	/// # use astral_thirdparty::slog;
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
	/// use astral::string::Name;
	///
	/// // some invalid bytes
	/// let input = b"Hello \xF0\x90\x80World";
	/// let output = Name::from_utf8_lossy(input, &string_subsystem);
	///
	/// assert_eq!("Hello �World", output);
	/// ```
	pub fn from_utf8_lossy(v: &[u8], system: &'system Subsystem<H>) -> Self {
		Self::new(String::from_utf8_lossy(v), system)
	}

	/// Converts a slice of bytes to a `Name` without checking that the
	/// string contains valid UTF-8.
	///
	/// See the safe version, [`from_utf8`], for more details.
	///
	/// [`from_utf8`]: #method.from_utf8
	///
	/// # Safety
	///
	/// This function is unsafe because it does not check that the bytes passed
	/// to it are valid UTF-8. If this constraint is violated, it may cause
	/// memory unsafety issues with future users of the `String`, as the rest of
	/// the library assumes that `Name`s are valid UTF-8.
	///
	/// # Example
	///
	/// ```
	/// # use astral_thirdparty::slog;
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
	/// use astral::string::Name;
	///
	/// // some bytes, in a vector
	/// let sparkle_heart = &[240, 159, 146, 150];
	///
	/// let sparkle_heart = unsafe {
	///     Name::from_utf8_unchecked(sparkle_heart, &string_subsystem)
	/// };
	///
	/// assert_eq!("💖", sparkle_heart);
	/// ```
	pub unsafe fn from_utf8_unchecked(v: &[u8], system: &'system Subsystem<H>) -> Self {
		Self::new(str::from_utf8_unchecked(v), system)
	}

	/// Decode a UTF-16 encoded slice into a `Name`, returning [`Err`]
	/// if the slice contains any invalid data.
	///
	/// # Example
	///
	/// ```
	/// # use astral_thirdparty::slog;
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
	/// use astral::string::Name;
	///
	/// // 𝄞music
	/// let v = &[0xD834, 0xDD1E, 0x006d, 0x0075,
	///           0x0073, 0x0069, 0x0063];
	/// assert_eq!(Name::new("𝄞music", &string_subsystem),
	///            Name::from_utf16(v, &string_subsystem).unwrap());
	///
	/// // 𝄞mu<invalid>ic
	/// let v = &[0xD834, 0xDD1E, 0x006d, 0x0075,
	///           0xD800, 0x0069, 0x0063];
	/// assert!(Name::from_utf16(v, &string_subsystem).is_err());
	/// ```
	pub fn from_utf16(v: &[u16], system: &'system Subsystem<H>) -> Result<Self, Utf16Error> {
		Ok(Self::new(
			String::from_utf16(v).map_err(Utf16Error::from_std)?,
			system,
		))
	}

	/// Decode a UTF-16 encoded slice into a `Name`, replacing
	/// invalid data with [the replacement character (`U+FFFD`)][U+FFFD].
	///
	/// [U+FFFD]: std::char::REPLACEMENT_CHARACTER
	///
	/// # Example
	///
	/// ```
	/// # use astral_thirdparty::slog;
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
	/// use astral::string::Name;
	///
	/// // 𝄞mus<invalid>ic<invalid>
	/// let v = &[0xD834, 0xDD1E, 0x006d, 0x0075,
	///           0x0073, 0xDD1E, 0x0069, 0x0063,
	///           0xD834];
	///
	/// assert_eq!(Name::new("𝄞mus\u{FFFD}ic\u{FFFD}", &string_subsystem),
	///            Name::from_utf16_lossy(v, &string_subsystem));
	/// ```
	pub fn from_utf16_lossy(v: &[u16], system: &'system Subsystem<H>) -> Self {
		Self::new(String::from_utf16_lossy(v), system)
	}
}
impl<'system, H> Name<'system, H> {
	fn split_string(string: &str) -> (&str, Option<NonZeroU32>) {
		let mut last_valid = None;
		for (index, byte) in string.bytes().enumerate().rev() {
			if byte.is_ascii_digit() {
				if byte != b'0' {
					last_valid = Some(index)
				}
			} else {
				break;
			}
		}
		last_valid.map_or((string, None), |idx| {
			let (prefix, number) = string.split_at(idx);
			u32::from_str(number)
				.map(|number| (prefix, Some(NonZeroU32::new(number).unwrap())))
				.unwrap_or((string, None))
		})
	}

	/// Creates a `Name` directly from a [`StringId`], and a number in the specified [`Subsystem`].
	///
	/// # Safety
	///
	/// The `Subsystem` must match the one, which were used to create the `StringId`.
	///
	/// # Example
	///
	/// ```
	/// # use astral_thirdparty::slog;
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
	/// use std::num::NonZeroU32;
	///
	/// use astral::string::{Name, StringId};
	///
	/// let id = StringId::new("Hello, world!", &string_subsystem);
	/// // safe because the subsystem is the same
	/// let hello = unsafe { Name::from_raw_parts(id, NonZeroU32::new(10), &string_subsystem) };
	///
	/// assert_eq!(hello, "Hello, world!10");
	/// ```
	///
	/// [`Subsystem`]: struct.Subsystem.html
	/// [`StringId`]: struct.StringId.html
	pub unsafe fn from_raw_parts(
		id: StringId,
		number: Option<NonZeroU32>,
		system: &'system Subsystem<H>,
	) -> Self {
		Self { id, number, system }
	}

	/// Returns the underlying [`StringId`].
	///
	/// The `StringId` will be the same, if the strings and the subsystem are equal or only differ
	/// at the numeric suffix.
	///
	/// # Example
	///
	/// ```
	/// # use astral_thirdparty::slog;
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
	/// use astral::string::Name;
	///
	/// let name1 = Name::new("foo-123", &string_subsystem);
	/// let name2 = Name::new("foo-456", &string_subsystem);
	///
	/// assert_ne!(name1, name2);
	/// assert_eq!(name1.id(), name2.id());
	/// ```
	///
	/// [`StringId`]: struct.StringId.html
	pub fn id(self) -> StringId {
		self.id
	}

	/// Returns the string part of the `Name`.
	///
	/// # Example
	///
	/// ```
	/// # use astral_thirdparty::slog;
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
	/// use astral::string::Name;
	///
	/// let s = Name::new("foo123", &string_subsystem);
	///
	/// assert_eq!("foo", s.string_part());
	/// ```
	pub fn string_part(self) -> &'system str {
		self.system.string(self.id)
	}

	/// Returns the number part of the `Name`.
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # use astral_thirdparty::slog;
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
	/// use astral::string::Name;
	///
	/// let s = Name::new("foo123", &string_subsystem);
	///
	/// assert_eq!(123, s.number().unwrap().get());
	/// ```
	pub fn number(self) -> Option<NonZeroU32> {
		self.number
	}

	/// Returns the string as [`Cow`].
	///
	/// If the `Name` does not contain a numeric suffix, a [`Borrowed`] can be returned. Otherwise,
	/// [`Owned`] is used.
	///
	/// [`Cow`]: std::borrow::Cow
	/// [`Borrowed`]: std::borrow::Cow::Borrowed
	/// [`Owned`]: std::borrow::Cow::Owned
	///
	/// # Example
	///
	/// ```
	/// # use astral_thirdparty::slog;
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
	/// use std::borrow::Cow;
	///
	/// use astral::string::Name;
	///
	/// let name = Name::new("foo", &string_subsystem);
	/// assert_eq!(name.as_str(), Cow::Borrowed("foo"));
	///
	/// let name = Name::new("bar-10", &string_subsystem);
	/// let cow: Cow<'_, str> = Cow::Owned(String::from("bar-10"));
	/// assert_eq!(name.as_str(), cow);
	/// ```
	///
	/// Remember, than a digital suffix with leading zeros cannot be optimized:
	///
	/// ```
	/// # use astral_thirdparty::slog;
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
	/// use std::borrow::Cow;
	///
	/// use astral::string::Name;
	///
	/// let name = Name::new("hello-010", &string_subsystem);
	/// assert_eq!(name.as_str(), Cow::Borrowed("hello-010"));
	/// ```
	pub fn as_str(self) -> Cow<'system, str> {
		if self.number.is_some() {
			Cow::Owned(self.to_string())
		} else {
			Cow::Borrowed(self.string_part())
		}
	}

	/// Returns `true` if this `Name` has a length of zero.
	///
	/// Returns `false` otherwise.
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # use astral_thirdparty::slog;
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
	/// use astral::string::Name;
	///
	/// let s = Name::new("foo", &string_subsystem);
	///
	/// assert!(!s.is_empty());
	/// assert!(Name::new("", &string_subsystem).is_empty());
	/// ```
	pub fn is_empty(self) -> bool {
		if self.number.is_some() {
			false
		} else {
			self.system.is_empty(self.id)
		}
	}

	/// Returns the length of this `Name`, in bytes.
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # use astral_thirdparty::slog;
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let string_subsystem = astral::string::Subsystem::new(64, &logger);
	/// use astral::string::Name;
	///
	/// let s = Name::new("foo", &string_subsystem);
	///
	/// assert_eq!(s.len(), 3);
	/// ```
	pub fn len(self) -> usize {
		let len = self.system.len(self.id);
		if let Some(number) = self.number() {
			len + number.to_string().len()
		} else {
			len
		}
	}
}

impl<H> Clone for Name<'_, H> {
	fn clone(&self) -> Self {
		unsafe { Self::from_raw_parts(self.id, self.number, self.system) }
	}
}

impl<H> Copy for Name<'_, H> {}

impl<B> Hash for Name<'_, B> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.id().hash(state);
		self.number().hash(state);
	}
}

#[allow(box_pointers)]
impl<H> From<Name<'_, H>> for Box<str> {
	fn from(string: Name<'_, H>) -> Self {
		string.to_string().into_boxed_str()
	}
}

impl<'system, H> From<Name<'system, H>> for Cow<'system, str> {
	#[inline]
	fn from(string: Name<'system, H>) -> Cow<'system, str> {
		string.as_str()
	}
}

impl<H> From<Name<'_, H>> for String {
	#[inline]
	fn from(string: Name<'_, H>) -> Self {
		string.to_string()
	}
}

impl<H> From<Name<'_, H>> for OsString {
	fn from(string: Name<'_, H>) -> Self {
		Self::from(string.to_string())
	}
}

impl<H> From<Name<'_, H>> for PathBuf {
	fn from(string: Name<'_, H>) -> Self {
		Self::from(string.to_string())
	}
}

#[allow(box_pointers)]
impl<H> From<Name<'_, H>> for Box<dyn Error> {
	fn from(string: Name<'_, H>) -> Self {
		Self::from(string.to_string())
	}
}

#[allow(box_pointers)]
impl<H> From<Name<'_, H>> for Box<dyn Error + Send + Sync> {
	fn from(string: Name<'_, H>) -> Self {
		Self::from(string.to_string())
	}
}

impl<H> Debug for Name<'_, H> {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		write!(fmt, "\"{}\"", self)
	}
}

impl<H> Display for Name<'_, H> {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		let string_part = self.string_part();
		if let Some(number) = self.number {
			write!(fmt, "{}{}", string_part, number)
		} else {
			Display::fmt(string_part, fmt)
		}
	}
}

impl<'system, H> Extend<Name<'system, H>> for String
where
	H: 'system,
{
	fn extend<I: IntoIterator<Item = Name<'system, H>>>(&mut self, iter: I) {
		for s in iter {
			self.push_str(&s.as_str())
		}
	}
}

impl<H> PartialEq for Name<'_, H> {
	#[inline]
	fn eq(&self, other: &Self) -> bool {
		let self_system: *const _ = &self.system;
		let other_system: *const _ = &other.system;
		if self_system == other_system {
			self.id == other.id && self.number() == other.number()
		} else {
			PartialEq::eq(
				&(self.string_part(), self.number),
				&(other.string_part(), other.number),
			)
		}
	}
}

impl<H> Eq for Name<'_, H> {}

impl<H> PartialOrd for Name<'_, H> {
	#[inline]
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		let self_system: *const _ = &self.system;
		let other_system: *const _ = &other.system;
		if self_system == other_system && self.id == other.id && self.number() == other.number() {
			Some(Ordering::Equal)
		} else {
			PartialOrd::partial_cmp(
				&(self.string_part(), self.number),
				&(other.string_part(), other.number),
			)
		}
	}
}

impl<H> Ord for Name<'_, H> {
	fn cmp(&self, other: &Self) -> Ordering {
		let self_system: *const _ = &self.system;
		let other_system: *const _ = &other.system;
		if self_system == other_system && self.id == other.id && self.number() == other.number() {
			Ordering::Equal
		} else {
			Ord::cmp(
				&(self.string_part(), self.number),
				&(other.string_part(), other.number),
			)
		}
	}
}

macro_rules! impl_cmp {
	($ty:ty) => {
		impl<H> PartialEq<$ty> for Name<'_, H> {
			#[inline]
			fn eq(&self, other: &$ty) -> bool {
				if self.number.is_some() {
					PartialEq::eq(
						&(self.string_part(), self.number),
						&Self::split_string(&other[..]),
					)
				} else {
					PartialEq::eq(self.string_part(), &other[..])
				}
			}
		}

		impl<H> PartialEq<Name<'_, H>> for $ty {
			#[inline]
			fn eq(&self, other: &Name<'_, H>) -> bool {
				if other.number.is_some() {
					PartialEq::eq(
						&Name::<'_, H>::split_string(&self[..]),
						&(other.string_part(), other.number),
					)
				} else {
					PartialEq::eq(&self[..], other.string_part())
				}
			}
		}

		impl<H> PartialOrd<$ty> for Name<'_, H> {
			#[inline]
			fn partial_cmp(&self, other: &$ty) -> Option<Ordering> {
				if self.number.is_some() {
					PartialOrd::partial_cmp(
						&(self.string_part(), self.number),
						&Self::split_string(&other[..]),
					)
				} else {
					PartialOrd::partial_cmp(self.string_part(), &other[..])
				}
			}
		}

		impl<H> PartialOrd<Name<'_, H>> for $ty {
			#[inline]
			fn partial_cmp(&self, other: &Name<'_, H>) -> Option<Ordering> {
				if other.number.is_some() {
					PartialOrd::partial_cmp(
						&Name::<'_, H>::split_string(&self[..]),
						&(other.string_part(), other.number),
					)
				} else {
					PartialOrd::partial_cmp(&self[..], other.string_part())
				}
			}
		}
	};
}

impl_cmp! { str }
impl_cmp! { &str }
impl_cmp! { String }
impl_cmp! { Cow<'_, str> }
impl_cmp! { Text<'_, H> }

#[cfg(test)]
mod test {
	#![allow(clippy::non_ascii_literal, clippy::shadow_unrelated)]

	use astral_thirdparty::slog;

	use super::*;

	#[cfg(target_pointer_width = "64")]
	#[test]
	fn test_size() {
		assert_eq!(std::mem::size_of::<Name<'_>>(), 16);
		assert_eq!(std::mem::size_of::<Option<Name<'_>>>(), 16);
	}

	#[cfg(target_pointer_width = "32")]
	#[test]
	fn test_size() {
		assert_eq!(std::mem::size_of::<Name<'_>>(), 12);
		assert_eq!(std::mem::size_of::<Option<Name<'_>>>(), 12);
	}

	#[test]
	fn test_from_utf8() {
		let logger = slog::Logger::root(slog::Discard, slog::o!());
		let string_subsystem = Subsystem::new(64, &logger);
		let xs = b"hello";
		assert_eq!(
			Name::from_utf8(xs, &string_subsystem).unwrap(),
			Name::new("hello", &string_subsystem)
		);

		let xs = "ศไทย中华Việt Nam".as_bytes();
		assert_eq!(
			Name::from_utf8(xs, &string_subsystem).unwrap(),
			Name::new("ศไทย中华Việt Nam", &string_subsystem)
		);
	}

	#[test]
	fn test_from_utf8_lossy() {
		let logger = slog::Logger::root(slog::Discard, slog::o!());
		let string_subsystem = Subsystem::new(64, &logger);
		let xs = b"hello";
		assert_eq!(Name::from_utf8_lossy(xs, &string_subsystem), "hello");

		let xs = "ศไทย中华Việt Nam".as_bytes();
		let ys = "ศไทย中华Việt Nam";
		assert_eq!(Name::from_utf8_lossy(xs, &string_subsystem), ys);

		let xs = b"Hello\xC2 There\xFF Goodbye";
		assert_eq!(
			Name::from_utf8_lossy(xs, &string_subsystem),
			Name::new("Hello\u{FFFD} There\u{FFFD} Goodbye", &string_subsystem)
		);

		let xs = b"Hello\xC0\x80 There\xE6\x83 Goodbye";
		assert_eq!(
			Name::from_utf8_lossy(xs, &string_subsystem),
			Name::new(
				"Hello\u{FFFD}\u{FFFD} There\u{FFFD} Goodbye",
				&string_subsystem
			)
		);

		let xs = b"\xF5foo\xF5\x80bar";
		assert_eq!(
			Name::from_utf8_lossy(xs, &string_subsystem),
			Name::new("\u{FFFD}foo\u{FFFD}\u{FFFD}bar", &string_subsystem)
		);

		let xs = b"\xF1foo\xF1\x80bar\xF1\x80\x80baz";
		assert_eq!(
			Name::from_utf8_lossy(xs, &string_subsystem),
			Name::new("\u{FFFD}foo\u{FFFD}bar\u{FFFD}baz", &string_subsystem)
		);

		let xs = b"\xF4foo\xF4\x80bar\xF4\xBFbaz";
		assert_eq!(
			Name::from_utf8_lossy(xs, &string_subsystem),
			Name::new(
				"\u{FFFD}foo\u{FFFD}bar\u{FFFD}\u{FFFD}baz",
				&string_subsystem
			)
		);

		let xs = b"\xF0\x80\x80\x80foo\xF0\x90\x80\x80bar";
		assert_eq!(
			Name::from_utf8_lossy(xs, &string_subsystem),
			Name::new(
				"\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}foo\u{10000}bar",
				&string_subsystem
			)
		);

		// surrogates
		let xs = b"\xED\xA0\x80foo\xED\xBF\xBFbar";
		assert_eq!(
			Name::from_utf8_lossy(xs, &string_subsystem),
			Name::new(
				"\u{FFFD}\u{FFFD}\u{FFFD}foo\u{FFFD}\u{FFFD}\u{FFFD}bar",
				&string_subsystem
			)
		);
	}

	#[test]
	fn test_from_utf16() {
		let logger = slog::Logger::root(slog::Discard, slog::o!());
		let string_subsystem = Subsystem::new(64, &logger);
		let pairs: [(Name<'_>, Vec<u16>); 5] = [(Name::new("𐍅𐌿𐌻𐍆𐌹𐌻𐌰\n", &string_subsystem),
                  vec![0xd800, 0xdf45, 0xd800, 0xdf3f, 0xd800, 0xdf3b, 0xd800, 0xdf46, 0xd800,
                       0xdf39, 0xd800, 0xdf3b, 0xd800, 0xdf30, 0x000a]),

                 (Name::new("𐐒𐑉𐐮𐑀𐐲𐑋 𐐏𐐲𐑍\n", &string_subsystem),
                  vec![0xd801, 0xdc12, 0xd801, 0xdc49, 0xd801, 0xdc2e, 0xd801, 0xdc40, 0xd801,
                       0xdc32, 0xd801, 0xdc4b, 0x0020, 0xd801, 0xdc0f, 0xd801, 0xdc32, 0xd801,
                       0xdc4d, 0x000a]),

                 (Name::new("𐌀𐌖𐌋𐌄𐌑𐌉·𐌌𐌄𐌕𐌄𐌋𐌉𐌑\n", &string_subsystem),
                  vec![0xd800, 0xdf00, 0xd800, 0xdf16, 0xd800, 0xdf0b, 0xd800, 0xdf04, 0xd800,
                       0xdf11, 0xd800, 0xdf09, 0x00b7, 0xd800, 0xdf0c, 0xd800, 0xdf04, 0xd800,
                       0xdf15, 0xd800, 0xdf04, 0xd800, 0xdf0b, 0xd800, 0xdf09, 0xd800, 0xdf11,
                       0x000a]),

                 (Name::new("𐒋𐒘𐒈𐒑𐒛𐒒 𐒕𐒓 𐒈𐒚𐒍 𐒏𐒜𐒒𐒖𐒆 𐒕𐒆\n", &string_subsystem),
                  vec![0xd801, 0xdc8b, 0xd801, 0xdc98, 0xd801, 0xdc88, 0xd801, 0xdc91, 0xd801,
                       0xdc9b, 0xd801, 0xdc92, 0x0020, 0xd801, 0xdc95, 0xd801, 0xdc93, 0x0020,
                       0xd801, 0xdc88, 0xd801, 0xdc9a, 0xd801, 0xdc8d, 0x0020, 0xd801, 0xdc8f,
                       0xd801, 0xdc9c, 0xd801, 0xdc92, 0xd801, 0xdc96, 0xd801, 0xdc86, 0x0020,
                       0xd801, 0xdc95, 0xd801, 0xdc86, 0x000a]),
                 (Name::new("\u{20000}", &string_subsystem), vec![0xD840, 0xDC00])];

		for p in &pairs {
			let (s, u) = (*p).clone();
			let s_str = s.as_str();
			let s_as_utf16 = s_str.encode_utf16().collect::<Vec<u16>>();
			let u_as_string = Name::from_utf16(&u, &string_subsystem).unwrap().as_str();

			assert!(std::char::decode_utf16(u.iter().cloned()).all(|r| r.is_ok()));
			assert_eq!(s_as_utf16, u);

			assert_eq!(u_as_string, s);
			assert_eq!(Name::from_utf16_lossy(&u, &string_subsystem), s);

			assert_eq!(Name::from_utf16(&s_as_utf16, &string_subsystem).unwrap(), s);
			assert_eq!(u_as_string.encode_utf16().collect::<Vec<u16>>(), u);
		}
	}

	#[test]
	fn test_utf16_invalid() {
		let logger = slog::Logger::root(slog::Discard, slog::o!());
		let string_subsystem = Subsystem::new(64, &logger);

		// completely positive cases tested above.
		// lead + eof
		assert!(Name::from_utf16(&[0xD800], &string_subsystem).is_err());
		// lead + lead
		assert!(Name::from_utf16(&[0xD800, 0xD800], &string_subsystem).is_err());

		// isolated trail
		assert!(Name::from_utf16(&[0x0061, 0xDC00], &string_subsystem).is_err());

		// general
		assert!(Name::from_utf16(&[0xD800, 0xd801, 0xdc8b, 0xD800], &string_subsystem).is_err());
	}

	#[test]
	fn test_from_utf16_lossy() {
		let logger = slog::Logger::root(slog::Discard, slog::o!());
		let string_subsystem = Subsystem::new(64, &logger);

		// completely positive cases tested above.
		// lead + eof
		assert_eq!(
			Name::from_utf16_lossy(&[0xD800], &string_subsystem),
			Name::new("\u{FFFD}", &string_subsystem)
		);
		// lead + lead
		assert_eq!(
			Name::from_utf16_lossy(&[0xD800, 0xD800], &string_subsystem),
			Name::new("\u{FFFD}\u{FFFD}", &string_subsystem)
		);

		// isolated trail
		assert_eq!(
			Name::from_utf16_lossy(&[0x0061, 0xDC00], &string_subsystem),
			Name::new("a\u{FFFD}", &string_subsystem)
		);

		// general
		assert_eq!(
			Name::from_utf16_lossy(&[0xD800, 0xd801, 0xdc8b, 0xD800], &string_subsystem),
			Name::new("\u{FFFD}𐒋\u{FFFD}", &string_subsystem)
		);
	}

	#[allow(clippy::string_extend_chars)]
	// Returning `mut` is allowed because of `UnsafeCell`
	#[test]
	fn test_from_iterator() {
		let logger = slog::Logger::root(slog::Discard, slog::o!());
		let string_subsystem = Subsystem::new(64, &logger);

		let s = Name::new("ศไทย中华Việt Nam", &string_subsystem);
		let t = "ศไทย中华";
		let u = "Việt Nam";

		let mut a = t.to_string();
		a.extend(u.chars());
		assert_eq!(s, a);

		let b: String = vec![t, u].into_iter().collect();
		assert_eq!(s, b);

		let mut c = t.to_string();
		c.extend(vec![u]);
		assert_eq!(s, c);
	}
}
