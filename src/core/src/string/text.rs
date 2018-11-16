// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::{
	borrow::{Borrow, Cow},
	cmp::{Ordering, PartialEq, PartialOrd},
	error::Error,
	ffi::{OsStr, OsString},
	fmt::{self, Debug, Display, Formatter},
	hash::{Hash, Hasher},
	num::NonZeroU32,
	ops::{
		Deref, Index, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo,
		RangeToInclusive,
	},
	path::{Path, PathBuf},
	str::{self, FromStr},
	string::ParseError,
};

use crate::hash::Murmur3;

use super::{Utf16Error, Utf8Error, ENTRY_HASH_TABLE, ENTRY_REFERENCE_MAP};

/// A UTF-8 encoded, immutable string.
///
/// # Examples
///
/// You can create a `Text` from a literal string with [`Text::from`]:
///
/// ```
/// # extern crate astral;
/// use astral::core::string::Text;
///
/// let hello = Text::from("Hello, world!");
/// ```
///
/// # Deref
///
/// `Text`s implement [`Deref`]`<Target=str>`, and so inherit all of [`str`]'s
/// methods. In addition, this means that you can pass a `Text` to a
/// function which takes a [`&str`][`str`] by using an ampersand (`&`):
///
/// ```
/// # extern crate astral;
/// use astral::core::string::Text;
///
/// fn takes_str(s: &str) { }
///
/// let s = Text::from("Hello");
///
/// takes_str(&s);
/// ```
///
/// # Representation
///
/// `Text` stores an index into a global table where the data is stored.
/// When a new `Text` is created, it is first checked if this `Text` already
/// exists. If so, it gets the same index as the existing one. If not, a
/// new entry is created in the table.
///
/// The index can be used to trivially check for equality and create a hash.
///
/// [`Text::from`]: #method.from
/// [`Deref`]: https://doc.rust-lang.org/nightly/std/ops/trait.Deref.html
/// [`str`]: https://doc.rust-lang.org/nightly/std/primitive.str.html
#[derive(Copy, Clone, Eq, PartialEq, Ord, Hash)]
pub struct Text {
	index: NonZeroU32,
}

impl Text {
	fn new(string: &str) -> Self {
		let mut hasher = Murmur3::default();
		Hash::hash_slice(string.as_bytes(), &mut hasher);

		Self {
			index: ENTRY_HASH_TABLE.find_or_insert(string, hasher.finish()),
		}
	}

	/// Converts a slice of bytes to a `Text`.
	///
	/// `Text` requires that it is valid UTF-8. `from_utf8()` checks to ensure
	/// that the bytes are valid UTF-8, and then does the conversion.
	///
	/// If you are sure that the byte slice is valid UTF-8, and you don't want to
	/// incur the overhead of the validity check, there is an unsafe version of
	/// this function, [`from_utf8_unchecked`], which has the same
	/// behavior but skips the check.
	///
	/// [`from_utf8_unchecked`]: struct.Text.html#method.from_utf8_unchecked
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
	/// use astral::core::string::Text;
	///
	/// // some bytes, in a vector
	/// let sparkle_heart = &[240, 159, 146, 150];
	///
	/// // We know these bytes are valid, so just use `unwrap()`.
	/// let sparkle_heart = Text::from_utf8(sparkle_heart).unwrap();
	///
	/// assert_eq!("💖", sparkle_heart);
	/// ```
	///
	/// Incorrect bytes:
	///
	/// ```
	/// # extern crate astral;
	/// use astral::core::string::Text;
	///
	/// // some invalid bytes, in a vector
	/// let sparkle_heart = &[0, 159, 146, 150];
	///
	/// assert!(Text::from_utf8(sparkle_heart).is_err());
	/// ```
	///
	/// See the docs for [`Utf8Error`][error] for more details on the kinds of
	/// errors that can be returned.
	///
	/// [error]: struct.Utf8Error.html
	pub fn from_utf8(v: &[u8]) -> Result<Self, Utf8Error> {
		Ok(Self::from(str::from_utf8(v).map_err(Utf8Error::from_std)?))
	}

	/// Converts a slice of bytes to a `Text`, including invalid characters.
	///
	/// `Text` requires that it is valid UTF-8. `from_utf8()` checks to ensure
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
	/// use astral::core::string::Text;
	///
	/// // some bytes, in a vector
	/// let sparkle_heart = vec![240, 159, 146, 150];
	///
	/// let sparkle_heart = Text::from_utf8_lossy(&sparkle_heart);
	///
	/// assert_eq!("💖", sparkle_heart);
	/// ```
	///
	/// Incorrect bytes:
	///
	/// ```
	/// # extern crate astral;
	/// use astral::core::string::Text;
	///
	/// // some invalid bytes
	/// let input = b"Hello \xF0\x90\x80World";
	/// let output = Text::from_utf8_lossy(input);
	///
	/// assert_eq!("Hello �World", output);
	/// ```
	pub fn from_utf8_lossy(v: &[u8]) -> Self {
		Self::from(String::from_utf8_lossy(v))
	}

	/// Converts a slice of bytes to a `Text` without checking that the
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
	/// the library assumes that `Text`s are valid UTF-8.
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # extern crate astral;
	/// use astral::core::string::Text;
	///
	/// // some bytes, in a vector
	/// let sparkle_heart = &[240, 159, 146, 150];
	///
	/// let sparkle_heart = unsafe {
	///     Text::from_utf8_unchecked(sparkle_heart)
	/// };
	///
	/// assert_eq!("💖", sparkle_heart);
	/// ```
	pub unsafe fn from_utf8_unchecked(v: &[u8]) -> Self {
		Self::from(str::from_utf8_unchecked(v))
	}

	/// Decode a UTF-16 encoded slice into a `Text`, returning [`Err`]
	/// if the slice contains any invalid data.
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # extern crate astral;
	/// use astral::core::string::Text;
	///
	/// // 𝄞music
	/// let v = &[0xD834, 0xDD1E, 0x006d, 0x0075,
	///           0x0073, 0x0069, 0x0063];
	/// assert_eq!(Text::from("𝄞music"),
	///            Text::from_utf16(v).unwrap());
	///
	/// // 𝄞mu<invalid>ic
	/// let v = &[0xD834, 0xDD1E, 0x006d, 0x0075,
	///           0xD800, 0x0069, 0x0063];
	/// assert!(Text::from_utf16(v).is_err());
	/// ```
	pub fn from_utf16(v: &[u16]) -> Result<Self, Utf16Error> {
		Ok(Self::from(
			String::from_utf16(v).map_err(Utf16Error::from_std)?,
		))
	}

	/// Decode a UTF-16 encoded slice into a `Text`, replacing
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
	/// use astral::core::string::Text;
	///
	/// // 𝄞mus<invalid>ic<invalid>
	/// let v = &[0xD834, 0xDD1E, 0x006d, 0x0075,
	///           0x0073, 0xDD1E, 0x0069, 0x0063,
	///           0xD834];
	///
	/// assert_eq!(Text::from("𝄞mus\u{FFFD}ic\u{FFFD}"),
	///            Text::from_utf16_lossy(v));
	/// ```
	pub fn from_utf16_lossy(v: &[u16]) -> Self {
		Self::from(String::from_utf16_lossy(v))
	}

	/// Extracts a string slice containing the entire `Text`.
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # extern crate astral;
	/// use astral::core::string::Text;
	///
	/// let s = Text::from("foo");
	///
	/// assert_eq!("foo", s.as_str());
	/// ```
	pub fn as_str(self) -> &'static str {
		debug_assert!(
			ENTRY_REFERENCE_MAP.get(self.index).is_some(),
			"invalid index"
		);
		unsafe { ENTRY_REFERENCE_MAP.get_unchecked(self.index).as_str() }
	}

	/// Returns `true` if this `Text` has a length of zero.
	///
	/// Returns `false` otherwise.
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # extern crate astral;
	/// use astral::core::string::Text;
	///
	/// assert!(Text::from("").is_empty());
	/// assert!(!Text::from("Hello World!").is_empty());
	/// ```
	pub fn is_empty(self) -> bool {
		debug_assert!(
			ENTRY_REFERENCE_MAP.get(self.index).is_some(),
			"invalid index"
		);
		unsafe {
			let entry = ENTRY_REFERENCE_MAP.get_unchecked(self.index);
			(*entry).is_empty()
		}
	}

	/// Returns the length of this `Text`, in bytes.
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # extern crate astral;
	/// use astral::core::string::Text;
	///
	/// let a = Text::from("foo");
	///
	/// assert_eq!(a.len(), 3);
	/// ```
	pub fn len(self) -> usize {
		debug_assert!(
			ENTRY_REFERENCE_MAP.get(self.index).is_some(),
			"invalid index"
		);
		unsafe {
			let entry = ENTRY_REFERENCE_MAP.get_unchecked(self.index);
			(*entry).len() as usize
		}
	}
}

impl Default for Text {
	fn default() -> Self {
		Self::from("")
	}
}

impl<'a> From<&'a str> for Text {
	fn from(string: &str) -> Self {
		Self::new(string)
	}
}

impl From<Text> for Box<str> {
	fn from(string: Text) -> Self {
		string.to_string().into_boxed_str()
	}
}

impl<'a> From<Cow<'a, str>> for Text {
	#[inline]
	fn from(string: Cow<'a, str>) -> Self {
		Self::from(&string[..])
	}
}

// TODO(#9): Use anonymous lifetimes
impl<'a> From<Text> for Cow<'a, str> {
	#[inline]
	fn from(string: Text) -> Cow<'static, str> {
		Cow::Borrowed(string.as_str())
	}
}

impl From<String> for Text {
	#[inline]
	fn from(string: String) -> Self {
		Self::from(&string[..])
	}
}

impl From<Text> for String {
	#[inline]
	fn from(string: Text) -> Self {
		string.to_string()
	}
}

impl From<Text> for OsString {
	fn from(string: Text) -> Self {
		Self::from(&string[..])
	}
}

impl From<Text> for PathBuf {
	fn from(string: Text) -> Self {
		Self::from(&string[..])
	}
}

impl From<Text> for Box<dyn Error> {
	fn from(string: Text) -> Self {
		Self::from(&string[..])
	}
}

impl From<Text> for Box<dyn Error + Send + Sync> {
	fn from(string: Text) -> Self {
		Self::from(&string[..])
	}
}

impl FromStr for Text {
	type Err = ParseError;

	#[inline]
	fn from_str(s: &str) -> Result<Self, ParseError> {
		Ok(Self::from(s))
	}
}

impl Deref for Text {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}

impl Debug for Text {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		Debug::fmt(&self[..], fmt)
	}
}

impl Display for Text {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self[..], fmt)
	}
}

impl Index<RangeFull> for Text {
	type Output = str;

	#[inline]
	fn index(&self, _index: RangeFull) -> &str {
		self.as_str()
	}
}

impl Index<Range<usize>> for Text {
	type Output = str;

	#[inline]
	fn index(&self, index: Range<usize>) -> &str {
		Index::index(&self[..], index)
	}
}

impl Index<RangeTo<usize>> for Text {
	type Output = str;

	#[inline]
	fn index(&self, index: RangeTo<usize>) -> &str {
		Index::index(&self[..], index)
	}
}

impl Index<RangeFrom<usize>> for Text {
	type Output = str;

	#[inline]
	fn index(&self, index: RangeFrom<usize>) -> &str {
		Index::index(&self[..], index)
	}
}

impl Index<RangeInclusive<usize>> for Text {
	type Output = str;

	#[inline]
	fn index(&self, index: RangeInclusive<usize>) -> &str {
		Index::index(&self[..], index)
	}
}

impl Index<RangeToInclusive<usize>> for Text {
	type Output = str;

	#[inline]
	fn index(&self, index: RangeToInclusive<usize>) -> &str {
		Index::index(&self[..], index)
	}
}

impl Borrow<str> for Text {
	#[inline]
	fn borrow(&self) -> &str {
		self
	}
}

impl AsRef<str> for Text {
	#[inline]
	fn as_ref(&self) -> &str {
		self
	}
}

impl AsRef<[u8]> for Text {
	#[inline]
	fn as_ref(&self) -> &[u8] {
		self.as_bytes()
	}
}

impl AsRef<OsStr> for Text {
	fn as_ref(&self) -> &OsStr {
		(&self[..]).as_ref()
	}
}

impl AsRef<Path> for Text {
	fn as_ref(&self) -> &Path {
		(&self[..]).as_ref()
	}
}

macro_rules! impl_cmp {
	($ty:ty) => {
		impl<'a> PartialEq<$ty> for Text {
			#[inline]
			fn eq(&self, other: &$ty) -> bool {
				PartialEq::eq(&self[..], &other[..])
			}
		}

		impl<'a> PartialEq<Text> for $ty {
			#[inline]
			fn eq(&self, other: &Text) -> bool {
				PartialEq::eq(&self[..], &other[..])
			}
		}

		impl<'a> PartialOrd<$ty> for Text {
			#[inline]
			fn partial_cmp(&self, other: &$ty) -> Option<Ordering> {
				PartialOrd::partial_cmp(&self[..], &other[..])
			}
		}

		impl<'a> PartialOrd<Text> for $ty {
			#[inline]
			fn partial_cmp(&self, other: &Text) -> Option<Ordering> {
				PartialOrd::partial_cmp(&self[..], &other[..])
			}
		}
	};
}

impl PartialOrd for Text {
	#[inline]
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		PartialOrd::partial_cmp(&self[..], &other[..])
	}
}

// TODO(#9): Use anonymous lifetimes
impl_cmp!{ str }
impl_cmp!{ &'a str }
impl_cmp!{ String }
impl_cmp!{ Cow<'a, str> }
