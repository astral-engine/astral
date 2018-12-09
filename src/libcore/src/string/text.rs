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
	iter::FromIterator,
	num::NonZeroU32,
	ops::{Deref, Index, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
	path::{Path, PathBuf},
	str::{self, FromStr},
	string::ParseError,
};

use crate::hash::Murmur3;

use super::{Name, Utf16Error, Utf8Error, ENTRY_HASH_TABLE, ENTRY_REFERENCE_MAP};

/// A UTF-8 encoded, immutable string.
///
/// # Examples
///
/// You can create a `Text` from a literal string with [`Text::from`]:
///
/// ```
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
/// [`Text::from`]: core::convert::From
/// [`Deref`]: core::ops::Deref
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
	/// [`from_utf8_unchecked`]: string::Name::from_utf8_unchecked
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

	/// use astral::core::string::Text;
	///
	/// // some invalid bytes, in a vector
	/// let sparkle_heart = &[0, 159, 146, 150];
	///
	/// assert!(Text::from_utf8(sparkle_heart).is_err());
	/// ```
	///
	/// See the docs for [`Utf8Error`] for more details on the kinds of
	/// errors that can be returned.
	///
	/// [`Utf8Error`]: string::Utf8Error
	pub fn from_utf8(v: &[u8]) -> Result<Self, Utf8Error> {
		Ok(Self::from(str::from_utf8(v).map_err(Utf8Error::from_std)?))
	}

	/// Converts a slice of bytes to a `Text`, including invalid characters.
	///
	/// `Text` requires that it is valid UTF-8. [`from_utf8`] checks to ensure
	/// that the bytes are valid UTF-8. During this conversion,
	/// `from_utf8_lossy` will replace any invalid UTF-8 sequences with
	/// [`U+FFFD REPLACEMENT CHARACTER`][U+FFFD], which looks like this: �
	///
	/// If you are sure that the byte slice is valid UTF-8, and you don't want
	/// to incur the overhead of the conversion, there is an unsafe version
	/// of this function, [`from_utf8_unchecked`], which has the same behavior
	/// but skips the checks.
	///
	/// [U+FFFD]: core::char::REPLACEMENT_CHARACTER
	/// [`from_utf8_unchecked`]: string::Name::from_utf8_unchecked
	/// [`from_utf8`]: string::Name::from_utf8
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```

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
	/// [`from_utf8`]: string::Name::from_utf8
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
	/// [U+FFFD]: core::char::REPLACEMENT_CHARACTER
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```

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

impl From<&str> for Text {
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

impl From<Text> for Cow<'_, str> {
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

impl Extend<Text> for String {
	fn extend<I: IntoIterator<Item = Text>>(&mut self, iter: I) {
		for s in iter {
			self.push_str(&s)
		}
	}
}

macro_rules! impl_from_iter {
	($lifetime:tt, $ty:ty) => {
		impl<$lifetime> FromIterator<$ty> for Text {
			fn from_iter<I: IntoIterator<Item = $ty>>(iter: I) -> Self {
				let mut buf = String::new();
				buf.extend(iter);
				buf.into()
			}
		}
	};
	($ty:ty) => {
		impl FromIterator<$ty> for Text {
			fn from_iter<I: IntoIterator<Item = $ty>>(iter: I) -> Self {
				let mut buf = String::new();
				buf.extend(iter);
				buf.into()
			}
		}
	};
}

impl_from_iter! { 'a, &'a str }
impl_from_iter! { char }
impl_from_iter! { String }
impl_from_iter! { 'a, Cow<'a, str> }
impl_from_iter! { Name }
impl_from_iter! { Text }

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
		impl PartialEq<$ty> for Text {
			#[inline]
			fn eq(&self, other: &$ty) -> bool {
				PartialEq::eq(&self[..], &other[..])
			}
		}

		impl PartialEq<Text> for $ty {
			#[inline]
			fn eq(&self, other: &Text) -> bool {
				PartialEq::eq(&self[..], &other[..])
			}
		}

		impl PartialOrd<$ty> for Text {
			#[inline]
			fn partial_cmp(&self, other: &$ty) -> Option<Ordering> {
				PartialOrd::partial_cmp(&self[..], &other[..])
			}
		}

		impl PartialOrd<Text> for $ty {
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

impl_cmp! { str }
impl_cmp! { &str }
impl_cmp! { String }
impl_cmp! { Cow<'_, str> }

#[cfg(test)]
mod test {
	#![allow(clippy::non_ascii_literal, clippy::shadow_unrelated)]

	use super::*;

	#[test]
	fn test_size() {
		assert_eq!(std::mem::size_of::<Text>(), 4);
		assert_eq!(std::mem::size_of::<Option<Text>>(), 4);
	}

	#[test]
	fn test_from_str() {
		let owned: Option<Text> = "string".parse().ok();
		assert_eq!(owned.as_ref().map(|s| s.deref()), Some("string"));
	}

	#[test]
	fn test_from_cow_str() {
		assert_eq!(Text::from(Cow::Borrowed("string")), "string");
		assert_eq!(Text::from(Cow::Owned(String::from("string"))), "string");
	}

	#[test]
	fn test_from_utf8() {
		let xs = b"hello";
		assert_eq!(Text::from_utf8(xs).unwrap(), Text::from("hello"));

		let xs = "ศไทย中华Việt Nam".as_bytes();
		assert_eq!(
			Text::from_utf8(xs).unwrap(),
			Text::from("ศไทย中华Việt Nam")
		);
	}

	#[test]
	fn test_from_utf8_lossy() {
		let xs = b"hello";
		assert_eq!(Text::from_utf8_lossy(xs), "hello");

		let xs = "ศไทย中华Việt Nam".as_bytes();
		let ys = "ศไทย中华Việt Nam";
		assert_eq!(Text::from_utf8_lossy(xs), ys);

		let xs = b"Hello\xC2 There\xFF Goodbye";
		assert_eq!(
			Text::from_utf8_lossy(xs),
			Text::from("Hello\u{FFFD} There\u{FFFD} Goodbye")
		);

		let xs = b"Hello\xC0\x80 There\xE6\x83 Goodbye";
		assert_eq!(
			Text::from_utf8_lossy(xs),
			Text::from("Hello\u{FFFD}\u{FFFD} There\u{FFFD} Goodbye")
		);

		let xs = b"\xF5foo\xF5\x80bar";
		assert_eq!(
			Text::from_utf8_lossy(xs),
			Text::from("\u{FFFD}foo\u{FFFD}\u{FFFD}bar")
		);

		let xs = b"\xF1foo\xF1\x80bar\xF1\x80\x80baz";
		assert_eq!(
			Text::from_utf8_lossy(xs),
			Text::from("\u{FFFD}foo\u{FFFD}bar\u{FFFD}baz")
		);

		let xs = b"\xF4foo\xF4\x80bar\xF4\xBFbaz";
		assert_eq!(
			Text::from_utf8_lossy(xs),
			Text::from("\u{FFFD}foo\u{FFFD}bar\u{FFFD}\u{FFFD}baz")
		);

		let xs = b"\xF0\x80\x80\x80foo\xF0\x90\x80\x80bar";
		assert_eq!(
			Text::from_utf8_lossy(xs),
			Text::from("\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}foo\u{10000}bar")
		);

		// surrogates
		let xs = b"\xED\xA0\x80foo\xED\xBF\xBFbar";
		assert_eq!(
			Text::from_utf8_lossy(xs),
			Text::from("\u{FFFD}\u{FFFD}\u{FFFD}foo\u{FFFD}\u{FFFD}\u{FFFD}bar")
		);
	}

	#[test]
	fn test_from_utf16() {
		let pairs: [(Text, Vec<u16>); 5] = [(Text::from("𐍅𐌿𐌻𐍆𐌹𐌻𐌰\n"),
                  vec![0xd800, 0xdf45, 0xd800, 0xdf3f, 0xd800, 0xdf3b, 0xd800, 0xdf46, 0xd800,
                       0xdf39, 0xd800, 0xdf3b, 0xd800, 0xdf30, 0x000a]),

                 (Text::from("𐐒𐑉𐐮𐑀𐐲𐑋 𐐏𐐲𐑍\n"),
                  vec![0xd801, 0xdc12, 0xd801, 0xdc49, 0xd801, 0xdc2e, 0xd801, 0xdc40, 0xd801,
                       0xdc32, 0xd801, 0xdc4b, 0x0020, 0xd801, 0xdc0f, 0xd801, 0xdc32, 0xd801,
                       0xdc4d, 0x000a]),

                 (Text::from("𐌀𐌖𐌋𐌄𐌑𐌉·𐌌𐌄𐌕𐌄𐌋𐌉𐌑\n"),
                  vec![0xd800, 0xdf00, 0xd800, 0xdf16, 0xd800, 0xdf0b, 0xd800, 0xdf04, 0xd800,
                       0xdf11, 0xd800, 0xdf09, 0x00b7, 0xd800, 0xdf0c, 0xd800, 0xdf04, 0xd800,
                       0xdf15, 0xd800, 0xdf04, 0xd800, 0xdf0b, 0xd800, 0xdf09, 0xd800, 0xdf11,
                       0x000a]),

                 (Text::from("𐒋𐒘𐒈𐒑𐒛𐒒 𐒕𐒓 𐒈𐒚𐒍 𐒏𐒜𐒒𐒖𐒆 𐒕𐒆\n"),
                  vec![0xd801, 0xdc8b, 0xd801, 0xdc98, 0xd801, 0xdc88, 0xd801, 0xdc91, 0xd801,
                       0xdc9b, 0xd801, 0xdc92, 0x0020, 0xd801, 0xdc95, 0xd801, 0xdc93, 0x0020,
                       0xd801, 0xdc88, 0xd801, 0xdc9a, 0xd801, 0xdc8d, 0x0020, 0xd801, 0xdc8f,
                       0xd801, 0xdc9c, 0xd801, 0xdc92, 0xd801, 0xdc96, 0xd801, 0xdc86, 0x0020,
                       0xd801, 0xdc95, 0xd801, 0xdc86, 0x000a]),
                 (Text::from("\u{20000}"), vec![0xD840, 0xDC00])];

		for p in &pairs {
			let (s, u) = (*p).clone();
			let s_str = s.as_str();
			let s_as_utf16 = s_str.encode_utf16().collect::<Vec<u16>>();
			let u_as_string = Text::from_utf16(&u).unwrap().as_str();

			assert!(std::char::decode_utf16(u.iter().cloned()).all(|r| r.is_ok()));
			assert_eq!(s_as_utf16, u);

			assert_eq!(u_as_string, s);
			assert_eq!(Text::from_utf16_lossy(&u), s);

			assert_eq!(Text::from_utf16(&s_as_utf16).unwrap(), s);
			assert_eq!(u_as_string.encode_utf16().collect::<Vec<u16>>(), u);
		}
	}

	#[test]
	fn test_utf16_invalid() {
		// completely positive cases tested above.
		// lead + eof
		assert!(Text::from_utf16(&[0xD800]).is_err());
		// lead + lead
		assert!(Text::from_utf16(&[0xD800, 0xD800]).is_err());

		// isolated trail
		assert!(Text::from_utf16(&[0x0061, 0xDC00]).is_err());

		// general
		assert!(Text::from_utf16(&[0xD800, 0xd801, 0xdc8b, 0xD800]).is_err());
	}

	#[test]
	fn test_from_utf16_lossy() {
		// completely positive cases tested above.
		// lead + eof
		assert_eq!(Text::from_utf16_lossy(&[0xD800]), Text::from("\u{FFFD}"));
		// lead + lead
		assert_eq!(
			Text::from_utf16_lossy(&[0xD800, 0xD800]),
			Text::from("\u{FFFD}\u{FFFD}")
		);

		// isolated trail
		assert_eq!(
			Text::from_utf16_lossy(&[0x0061, 0xDC00]),
			Text::from("a\u{FFFD}")
		);

		// general
		assert_eq!(
			Text::from_utf16_lossy(&[0xD800, 0xd801, 0xdc8b, 0xD800]),
			Text::from("\u{FFFD}𐒋\u{FFFD}")
		);
	}

	// Returning `mut` is allowed because of `UnsafeCell`
	#[allow(clippy::string_extend_chars)]
	#[test]
	fn test_from_iterator() {
		let s = Text::from("ศไทย中华Việt Nam");
		let t = "ศไทย中华";
		let u = "Việt Nam";

		let a: Text = s.chars().collect();
		assert_eq!(s, a);

		let mut b = t.to_string();
		b.extend(u.chars());
		assert_eq!(s, b);

		let c: String = vec![t, u].into_iter().collect();
		assert_eq!(s, c);

		let mut d = t.to_string();
		d.extend(vec![u]);
		assert_eq!(s, d);
	}

}
