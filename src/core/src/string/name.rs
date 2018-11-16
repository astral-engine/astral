// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::{
	borrow::Cow,
	cmp::{Ordering, PartialEq, PartialOrd},
	error::Error,
	ffi::OsString,
	fmt::{self, Debug, Display, Formatter},
	hash::{Hash, Hasher},
	num::NonZeroU32,
	path::PathBuf,
	str::{self, FromStr},
	string::ParseError,
};

use crate::hash::Murmur3;

use super::{
	Text, Utf16Error, Utf8Error, ENTRY_HASH_TABLE, ENTRY_REFERENCE_MAP,
};

/// A UTF-8 encoded, immutable string optimized for numeric suffixes.
///
/// # Examples
///
/// You can create a `Name` from a literal string with [`Name::from`]:
///
/// ```
/// # extern crate astral;
/// use astral::core::string::Name;
///
/// let hello = Name::from("Hello, world!");
/// ```
///
/// # Representation
///
/// `Name` stores an index into a global table where the data is stored and
/// an optional numeric suffix.
/// When a new `Name` is created, it is first checked if this `Name` already
/// exists. If so, it gets the same index as the existing one. If not, a
/// new entry is created in the table.
/// The suffix is only used for reusing the same string multiple times when
/// the string only differs at a numeric suffix. A suffix with leading zeros
/// cannot be optimized!
///
/// The index can be used to trivially check for equality and create a hash.
///
/// [`Name::from`]: #method.from
/// [`Deref`]: https://doc.rust-lang.org/nightly/std/ops/trait.Deref.html
/// [`str`]: https://doc.rust-lang.org/nightly/std/primitive.str.html
#[derive(Copy, Clone, Eq, PartialEq, Ord, Hash)]
pub struct Name {
	index: NonZeroU32,
	number: Option<NonZeroU32>,
}

impl Name {
	fn new(string: &str) -> Self {
		let (string, number) = Self::split_string(string);

		let mut hasher = Murmur3::default();
		Hash::hash_slice(string.as_bytes(), &mut hasher);
		Self {
			index: ENTRY_HASH_TABLE.find_or_insert(string, hasher.finish()),
			number,
		}
	}

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

	fn string_part(self) -> &'static str {
		unsafe { ENTRY_REFERENCE_MAP.get_unchecked(self.index).as_str() }
	}

	/// Returns the string as [`Cow`]`<'static, `[`str`]`>`.
	///
	/// If the `Name` does not contain a numeric suffix, a [`&'static str`][`str`]
	/// can be returned. Otherwise, a [`String`] is constructed.
	///
	/// [`Cow`]: https://doc.rust-lang.org/nightly/std/borrow/enum.Cow.html
	/// [`str`]: https://doc.rust-lang.org/nightly/std/primitive.str.html
	///
	/// # Example
	///
	/// ```
	/// # extern crate astral;
	/// use std::borrow::Cow;
	///
	/// use astral::core::string::Name;
	///
	/// let name = Name::from("foo");
	/// assert_eq!(name.as_str(), Cow::Borrowed("foo"));
	///
	/// let name = Name::from("bar-10");
	/// let cow: Cow<str> = Cow::Owned(String::from("bar-10"));
	/// assert_eq!(name.as_str(), cow);
	/// ```
	///
	/// Remember, than a digital suffix with leading zeros cannot be optimized:
	///
	/// ```
	/// # extern crate astral;
	/// use std::borrow::Cow;
	///
	/// use astral::core::string::Name;
	///
	/// let name = Name::from("hello-010");
	/// assert_eq!(name.as_str(), Cow::Borrowed("hello-010"));
	/// ```
	pub fn as_str(self) -> Cow<'static, str> {
		if self.number.is_some() {
			Cow::Owned(self.to_string())
		} else {
			Cow::Borrowed(self.string_part())
		}
	}

	/// Converts a slice of bytes to a `Name`.
	///
	/// `Name` requires that it is valid UTF-8. `from_utf8()` checks to ensure
	/// that the bytes are valid UTF-8, and then does the conversion.
	///
	/// If you are sure that the byte slice is valid UTF-8, and you don't want to
	/// incur the overhead of the validity check, there is an unsafe version of
	/// this function, [`from_utf8_unchecked`], which has the same
	/// behavior but skips the check.
	///
	/// [`from_utf8_unchecked`]: struct.Name.html#method.from_utf8_unchecked
	///
	/// # Errors
	///
	/// Returns [`Err`] if the slice is not UTF-8 with a description as to why the
	/// provided slice is not UTF-8.
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # extern crate astral;
	/// use astral::core::string::Name;
	///
	/// // some bytes, in a vector
	/// let sparkle_heart = &[240, 159, 146, 150];
	///
	/// // We know these bytes are valid, so just use `unwrap()`.
	/// let sparkle_heart = Name::from_utf8(sparkle_heart).unwrap();
	///
	/// assert_eq!("💖", sparkle_heart);
	/// ```
	///
	/// Incorrect bytes:
	///
	/// ```
	/// # extern crate astral;
	/// use astral::core::string::Name;
	///
	/// // some invalid bytes, in a vector
	/// let sparkle_heart = &[0, 159, 146, 150];
	///
	/// assert!(Name::from_utf8(sparkle_heart).is_err());
	/// ```
	///
	/// See the docs for [`Utf8Error`][error] for more details on the kinds of
	/// errors that can be returned.
	///
	/// [error]: struct.Utf8Error.html
	pub fn from_utf8(v: &[u8]) -> Result<Self, Utf8Error> {
		Ok(Self::from(str::from_utf8(v).map_err(Utf8Error::from_std)?))
	}

	/// Converts a slice of bytes to a `Name`, including invalid characters.
	///
	/// `Name` requires that it is valid UTF-8. `from_utf8()` checks to ensure
	/// that the bytes are valid UTF-8. During this conversion,
	/// `from_utf8_lossy()` will replace any invalid UTF-8 sequences with
	/// [`U+FFFD REPLACEMENT CHARACTER`][U+FFFD], which looks like this: �
	///
	/// [U+FFFD]: https://doc.rust-lang.org/nightly/std/char/constant.REPLACEMENT_CHARACTER.html
	///
	/// If you are sure that the byte slice is valid UTF-8, and you don't want
	/// to incur the overhead of the conversion, there is an unsafe version
	/// of this function, [`from_utf8_unchecked`], which has the same behavior
	/// but skips the checks.
	///
	/// [`from_utf8_unchecked`]: #method.from_utf8_unchecked
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # extern crate astral;
	/// use astral::core::string::Name;
	///
	/// // some bytes, in a vector
	/// let sparkle_heart = vec![240, 159, 146, 150];
	///
	/// let sparkle_heart = Name::from_utf8_lossy(&sparkle_heart);
	///
	/// assert_eq!("💖", sparkle_heart);
	/// ```
	///
	/// Incorrect bytes:
	///
	/// ```
	/// # extern crate astral;
	/// use astral::core::string::Name;
	///
	/// // some invalid bytes
	/// let input = b"Hello \xF0\x90\x80World";
	/// let output = Name::from_utf8_lossy(input);
	///
	/// assert_eq!("Hello �World", output);
	/// ```
	pub fn from_utf8_lossy(v: &[u8]) -> Self {
		Self::from(String::from_utf8_lossy(v))
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
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # extern crate astral;
	/// use astral::core::string::Name;
	///
	/// // some bytes, in a vector
	/// let sparkle_heart = &[240, 159, 146, 150];
	///
	/// let sparkle_heart = unsafe {
	///     Name::from_utf8_unchecked(sparkle_heart)
	/// };
	///
	/// assert_eq!("💖", sparkle_heart);
	/// ```
	pub unsafe fn from_utf8_unchecked(v: &[u8]) -> Self {
		Self::from(str::from_utf8_unchecked(v))
	}

	/// Decode a UTF-16 encoded slice into a `Name`, returning [`Err`]
	/// if the slice contains any invalid data.
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # extern crate astral;
	/// use astral::core::string::Name;
	///
	/// // 𝄞music
	/// let v = &[0xD834, 0xDD1E, 0x006d, 0x0075,
	///           0x0073, 0x0069, 0x0063];
	/// assert_eq!(Name::from("𝄞music"),
	///            Name::from_utf16(v).unwrap());
	///
	/// // 𝄞mu<invalid>ic
	/// let v = &[0xD834, 0xDD1E, 0x006d, 0x0075,
	///           0xD800, 0x0069, 0x0063];
	/// assert!(Name::from_utf16(v).is_err());
	/// ```
	pub fn from_utf16(v: &[u16]) -> Result<Self, Utf16Error> {
		Ok(Self::from(
			String::from_utf16(v).map_err(Utf16Error::from_std)?,
		))
	}

	/// Decode a UTF-16 encoded slice into a `Name`, replacing
	/// invalid data with [the replacement character (`U+FFFD`)][U+FFFD].
	///
	/// [U+FFFD]: https://doc.rust-lang.org/nightly/std/char/constant.REPLACEMENT_CHARACTER.html
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # extern crate astral;
	/// use astral::core::string::Name;
	///
	/// // 𝄞mus<invalid>ic<invalid>
	/// let v = &[0xD834, 0xDD1E, 0x006d, 0x0075,
	///           0x0073, 0xDD1E, 0x0069, 0x0063,
	///           0xD834];
	///
	/// assert_eq!(Name::from("𝄞mus\u{FFFD}ic\u{FFFD}"),
	///            Name::from_utf16_lossy(v));
	/// ```
	pub fn from_utf16_lossy(v: &[u16]) -> Self {
		Self::from(String::from_utf16_lossy(v))
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
	/// # extern crate astral;
	/// use astral::core::string::Name;
	///
	/// assert!(Name::from("").is_empty());
	/// assert!(!Name::from("Hello World!").is_empty());
	/// ```
	pub fn is_empty(self) -> bool {
		if self.number.is_some() {
			false
		} else {
			debug_assert!(
				ENTRY_REFERENCE_MAP.get(self.index).is_some(),
				"invalid index"
			);
			unsafe {
				let entry = ENTRY_REFERENCE_MAP.get_unchecked(self.index);
				(*entry).is_empty()
			}
		}
	}

	/// Returns the length of this `Name`, in bytes.
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # extern crate astral;
	/// use astral::core::string::Name;
	///
	/// let a = Name::from("foo");
	///
	/// assert_eq!(a.len(), 3);
	/// ```
	pub fn len(self) -> usize {
		let length = self.string_part().len();
		if let Some(number) = self.number {
			length + number.to_string().len()
		} else {
			length
		}
	}
}

impl Default for Name {
	fn default() -> Self {
		Self::from("")
	}
}

impl<'a> From<&'a str> for Name {
	fn from(string: &str) -> Self {
		Self::new(string)
	}
}

impl From<Text> for Name {
	#[inline]
	fn from(string: Text) -> Self {
		Self::from(&string[..])
	}
}

impl From<Name> for Text {
	#[inline]
	fn from(string: Name) -> Self {
		Self::from(string.as_str())
	}
}

impl From<Name> for Box<str> {
	fn from(string: Name) -> Self {
		string.to_string().into_boxed_str()
	}
}

impl<'a> From<Cow<'a, str>> for Name {
	#[inline]
	fn from(string: Cow<'a, str>) -> Self {
		Self::from(&string[..])
	}
}

// TODO(#9): Use anonymous lifetimes
impl<'a> From<Name> for Cow<'a, str> {
	#[inline]
	fn from(string: Name) -> Cow<'static, str> {
		string.as_str()
	}
}

impl From<String> for Name {
	#[inline]
	fn from(string: String) -> Self {
		Self::from(&string[..])
	}
}

impl From<Name> for String {
	#[inline]
	fn from(string: Name) -> Self {
		string.to_string()
	}
}

impl From<Name> for OsString {
	fn from(string: Name) -> Self {
		Self::from(string.to_string())
	}
}

impl From<Name> for PathBuf {
	fn from(string: Name) -> Self {
		Self::from(string.to_string())
	}
}

impl From<Name> for Box<dyn Error> {
	fn from(string: Name) -> Self {
		Self::from(string.to_string())
	}
}

impl From<Name> for Box<dyn Error + Send + Sync> {
	fn from(string: Name) -> Self {
		Self::from(string.to_string())
	}
}

impl FromStr for Name {
	type Err = ParseError;

	#[inline]
	fn from_str(s: &str) -> Result<Self, ParseError> {
		Ok(Self::from(s))
	}
}

impl Debug for Name {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		Debug::fmt(self.as_str().as_ref(), fmt)
	}
}

impl Display for Name {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		let string_part = self.string_part();
		if let Some(number) = self.number {
			write!(fmt, "{}{}", string_part, number)
		} else {
			Display::fmt(string_part, fmt)
		}
	}
}

macro_rules! impl_cmp {
	($ty:ty) => {
		impl<'a> PartialEq<$ty> for Name {
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

		impl<'a> PartialEq<Name> for $ty {
			#[inline]
			fn eq(&self, other: &Name) -> bool {
				if other.number.is_some() {
					PartialEq::eq(
						&Name::split_string(&self[..]),
						&(other.string_part(), other.number),
					)
				} else {
					PartialEq::eq(&self[..], other.string_part())
				}
			}
		}

		impl<'a> PartialOrd<$ty> for Name {
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

		impl<'a> PartialOrd<Name> for $ty {
			#[inline]
			fn partial_cmp(&self, other: &Name) -> Option<Ordering> {
				if other.number.is_some() {
					PartialOrd::partial_cmp(
						&Name::split_string(&self[..]),
						&(other.string_part(), other.number),
					)
				} else {
					PartialOrd::partial_cmp(&self[..], other.string_part())
				}
			}
		}
	};
}

impl PartialOrd for Name {
	#[inline]
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		PartialOrd::partial_cmp(
			&(self.string_part(), self.number),
			&(other.string_part(), other.number),
		)
	}
}

// TODO(#9): Use anonymous lifetimes
impl_cmp!{ str }
impl_cmp!{ &'a str }
impl_cmp!{ String }
impl_cmp!{ Cow<'a, str> }
impl_cmp!{ Text }
