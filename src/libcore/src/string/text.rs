// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::{
	borrow::{Borrow, Cow},
	cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd},
	error::Error,
	ffi::{OsStr, OsString},
	fmt::{self, Debug, Display, Formatter},
	hash::{BuildHasher, BuildHasherDefault, Hash, Hasher},
	ops::{Deref, Index, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
	path::{Path, PathBuf},
	str,
};

use crate::hash::Murmur3;

use super::{Name, StringId, Subsystem, Utf16Error, Utf8Error};

/// A UTF-8 encoded, immutable string.
///
/// # Examples
///
/// `Text` can be created from a literal string:
///
/// ```
/// # use astral::{third_party::slog, Engine, core::{System, string}};
///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
///	# let engine = Engine::new(&logger);
///	# let system = System::new(&engine);
///	# let string_subsystem = string::Subsystem::new(64, &system);
/// use astral::core::string::Text;
///
/// let text = Text::new("foo", &string_subsystem);
/// assert_eq!(text, "foo");
/// ```
///
/// ### Deref
///
/// `Text`s implement [`Deref`]`<Target=str>`, and so inherit all of [`str`]'s
/// methods. In addition, this means that you can pass a `Text` to a
/// function which takes a [`&str`][`str`] by using an ampersand (`&`):
///
/// ```
/// # use astral::{third_party::slog, Engine, core::{System, string}};
///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
///	# let engine = Engine::new(&logger);
///	# let system = System::new(&engine);
///	# let string_subsystem = string::Subsystem::new(64, &system);
/// use astral::core::string::Text;
///
/// # #[allow(unused_variables)]
/// fn takes_str(s: &str) { }
///
/// let s = Text::new("Hello", &string_subsystem);
///
/// takes_str(&s);
/// ```
///
/// # Representation
///
/// `Name` stores a [`StringId`], and a reference to a [`Subsystem`]. When a new `Text` is created,
/// it is first checked if the string already exists. If so, it gets the same index as the existing
/// one. If not, a new entry is created.
///
/// The [`StringId`] can be used to trivially check for equality.
///
/// [`StringId`]: astral_core::string::StringId
/// [`Subsystem`]: astral_core::string::Subsystem
/// [`Deref`]: std::ops::Deref
pub struct Text<'system, H = BuildHasherDefault<Murmur3>> {
	id: StringId,
	system: &'system Subsystem<H>,
}

impl<'system, H> Text<'system, H>
where
	H: BuildHasher,
{
	/// Creates a `Text` from the given string literal in the specified [`Subsystem`].
	///
	/// [`Subsystem`]: astral_core::string::Subsystem
	///
	/// # Example
	///
	/// ```
	/// # use astral::{third_party::slog, Engine, core::{System, string}};
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let system = System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &system);
	/// use astral::core::string::Name;
	///
	/// let name = Name::new("foo", &string_subsystem);
	/// assert_eq!(name, name);
	/// ```
	pub fn new<T>(string: T, system: &'system Subsystem<H>) -> Self
	where
		T: AsRef<str>,
	{
		unsafe { Self::from_raw_parts(system.create_string_id(string), system) }
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
	/// [`from_utf8_unchecked`]: astral_core::string::Text::from_utf8_unchecked
	///
	/// # Errors
	///
	/// Returns [`Err`] if the slice is not UTF-8 with a description as to why the
	/// provided slice is not UTF-8.
	///
	/// See the docs for [`Utf8Error`] for more details on the kinds of
	/// errors that can be returned.
	///
	/// [`Utf8Error`]: astral_core::string::Utf8Error
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # use astral::{third_party::slog, Engine, core::{System, string}};
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let system = System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &system);
	/// use astral::core::string::Text;
	///
	/// // some bytes, in a vector
	/// let sparkle_heart = &[240, 159, 146, 150];
	///
	/// // We know these bytes are valid, so just use `unwrap()`.
	/// let sparkle_heart = Text::from_utf8(sparkle_heart, &string_subsystem).unwrap();
	///
	/// assert_eq!("💖", sparkle_heart);
	/// ```
	///
	/// Incorrect bytes:
	///
	/// ```
	/// # use astral::{third_party::slog, Engine, core::{System, string}};
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let system = System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &system);
	/// use astral::core::string::Text;
	///
	/// // some invalid bytes, in a vector
	/// let sparkle_heart = &[0, 159, 146, 150];
	///
	/// assert!(Text::from_utf8(sparkle_heart, &string_subsystem).is_err());
	/// ```
	///
	/// See the docs for [`Utf8Error`] for more details on the kinds of
	/// errors that can be returned.
	///
	/// [`Utf8Error`]: astral_core::string::Utf8Error
	pub fn from_utf8(v: &[u8], system: &'system Subsystem<H>) -> Result<Self, Utf8Error> {
		Ok(Self::new(
			str::from_utf8(v).map_err(Utf8Error::from_std)?,
			system,
		))
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
	/// [U+FFFD]: std::char::REPLACEMENT_CHARACTER
	/// [`from_utf8_unchecked`]: astral_core::string::Text::from_utf8_unchecked
	/// [`from_utf8`]: astral_core::string::Text::from_utf8
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # use astral::{third_party::slog, Engine, core::{System, string}};
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let system = System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &system);
	/// use astral::core::string::Text;
	///
	/// // some bytes, in a vector
	/// let sparkle_heart = vec![240, 159, 146, 150];
	///
	/// let sparkle_heart = Text::from_utf8_lossy(&sparkle_heart, &string_subsystem);
	///
	/// assert_eq!("💖", sparkle_heart);
	/// ```
	///
	/// Incorrect bytes:
	///
	/// ```
	/// # use astral::{third_party::slog, Engine, core::{System, string}};
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let system = System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &system);
	/// use astral::core::string::Text;
	///
	/// // some invalid bytes
	/// let input = b"Hello \xF0\x90\x80World";
	/// let output = Text::from_utf8_lossy(input, &string_subsystem);
	///
	/// assert_eq!("Hello �World", output);
	/// ```
	pub fn from_utf8_lossy(v: &[u8], system: &'system Subsystem<H>) -> Self {
		Self::new(String::from_utf8_lossy(v), system)
	}

	/// Converts a slice of bytes to a `Text` without checking that the
	/// string contains valid UTF-8.
	///
	/// See the safe version, [`from_utf8`], for more details.
	///
	/// [`from_utf8`]: astral_core::string::Text::from_utf8
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
	/// # use astral::{third_party::slog, Engine, core::{System, string}};
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let system = System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &system);
	/// use astral::core::string::Text;
	///
	/// // some bytes, in a vector
	/// let sparkle_heart = &[240, 159, 146, 150];
	///
	/// let sparkle_heart = unsafe {
	///     Text::from_utf8_unchecked(sparkle_heart, &string_subsystem)
	/// };
	///
	/// assert_eq!("💖", sparkle_heart);
	/// ```
	pub unsafe fn from_utf8_unchecked(v: &[u8], system: &'system Subsystem<H>) -> Self {
		Self::new(str::from_utf8_unchecked(v), system)
	}

	/// Decode a UTF-16 encoded slice into a `Text`, returning [`Err`]
	/// if the slice contains any invalid data.
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # use astral::{third_party::slog, Engine, core::{System, string}};
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let system = System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &system);
	/// use astral::core::string::Text;
	///
	/// // 𝄞music
	/// let v = &[0xD834, 0xDD1E, 0x006d, 0x0075,
	///           0x0073, 0x0069, 0x0063];
	/// assert_eq!(Text::new("𝄞music", &string_subsystem),
	///            Text::from_utf16(v, &string_subsystem).unwrap());
	///
	/// // 𝄞mu<invalid>ic
	/// let v = &[0xD834, 0xDD1E, 0x006d, 0x0075,
	///           0xD800, 0x0069, 0x0063];
	/// assert!(Text::from_utf16(v, &string_subsystem).is_err());
	/// ```
	pub fn from_utf16(v: &[u16], system: &'system Subsystem<H>) -> Result<Self, Utf16Error> {
		Ok(Self::new(
			String::from_utf16(v).map_err(Utf16Error::from_std)?,
			system,
		))
	}

	/// Decode a UTF-16 encoded slice into a `Text`, replacing
	/// invalid data with [the replacement character (`U+FFFD`)][U+FFFD].
	///
	/// [U+FFFD]: std::char::REPLACEMENT_CHARACTER
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # use astral::{third_party::slog, Engine, core::{System, string}};
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let system = System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &system);
	/// use astral::core::string::Text;
	///
	/// // 𝄞mus<invalid>ic<invalid>
	/// let v = &[0xD834, 0xDD1E, 0x006d, 0x0075,
	///           0x0073, 0xDD1E, 0x0069, 0x0063,
	///           0xD834];
	///
	/// assert_eq!(Text::new("𝄞mus\u{FFFD}ic\u{FFFD}", &string_subsystem),
	///            Text::from_utf16_lossy(v, &string_subsystem));
	/// ```
	pub fn from_utf16_lossy(v: &[u16], system: &'system Subsystem<H>) -> Self {
		Self::new(String::from_utf16_lossy(v), system)
	}
}
impl<'system, H> Text<'system, H> {
	/// Creates a `Text` directly from a [`StringId`] in the specified [`Subsystem`].
	///
	/// # Safety
	///
	/// The `Subsystem` must match the one, which were used to create the `StringId`.
	///
	/// # Example
	///
	/// ```
	/// # use astral::{third_party::slog, Engine, core::{System, string}};
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let system = System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &system);
	/// use astral::core::string::{Text, StringId};
	///
	/// let id = StringId::new("Hello, world!", &string_subsystem);
	/// // safe because the subsystem is the same
	/// let hello = unsafe { Text::from_raw_parts(id, &string_subsystem) };
	///
	/// assert_eq!(hello, "Hello, world!");
	/// ```
	///
	/// [`Subsystem`]: astral_core::string::Subsystem
	pub unsafe fn from_raw_parts(id: StringId, system: &'system Subsystem<H>) -> Self {
		Self { id, system }
	}

	/// Returns the underlying [`StringId`].
	///
	/// The `StringId` will be the same, if the strings and the subsystem are equal .
	///
	/// # Example
	///
	/// ```
	/// # use astral::{third_party::slog, Engine, core::{System, string}};
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let system = System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &system);
	/// use astral::core::string::Text;
	///
	/// let text1 = Text::new("foo", &string_subsystem);
	/// let text2 = Text::new("foo", &string_subsystem);
	///
	/// assert_eq!(text1.id(), text2.id());
	/// ```
	///
	/// [`StringId`]: astral_core::string::StringId
	pub fn id(self) -> StringId {
		self.id
	}

	/// Extracts a string slice containing the entire `Text`.
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # use astral::{third_party::slog, Engine, core::{System, string}};
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let system = System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &system);
	/// use astral::core::string::Text;
	///
	/// let s = Text::new("foo", &string_subsystem);
	///
	/// assert_eq!("foo", s.as_str());
	/// ```
	pub fn as_str(self) -> &'system str {
		self.system.string(self.id)
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
	/// # use astral::{third_party::slog, Engine, core::{System, string}};
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let system = System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &system);
	/// use astral::core::string::Text;
	///
	/// let s = Text::new("foo", &string_subsystem);
	///
	/// assert!(!s.is_empty());
	/// assert!(Text::new("", &string_subsystem).is_empty());
	/// ```
	pub fn is_empty(self) -> bool {
		self.system.is_empty(self.id)
	}

	/// Returns the length of this `Text`, in bytes.
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// # use astral::{third_party::slog, Engine, core::{System, string}};
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let system = System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &system);
	/// use astral::core::string::Text;
	///
	/// let s = Text::new("foo", &string_subsystem);
	///
	/// assert_eq!(s.len(), 3);
	/// ```
	pub fn len(self) -> usize {
		self.system.len(self.id)
	}
}

impl<H> Clone for Text<'_, H> {
	fn clone(&self) -> Self {
		unsafe { Self::from_raw_parts(self.id, self.system) }
	}
}

impl<H> Copy for Text<'_, H> {}

impl<B> Hash for Text<'_, B> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.id().hash(state);
	}
}

#[allow(box_pointers)]
impl<H> From<Text<'_, H>> for Box<str> {
	fn from(string: Text<'_, H>) -> Self {
		string.to_string().into_boxed_str()
	}
}

impl<'system, H> From<Text<'system, H>> for Name<'system, H> {
	#[inline]
	fn from(text: Text<'system, H>) -> Self {
		unsafe { Self::from_raw_parts(text.id, None, text.system) }
	}
}

impl<'system, H> From<Text<'system, H>> for Cow<'system, str> {
	#[inline]
	fn from(string: Text<'system, H>) -> Cow<'system, str> {
		Cow::Borrowed(string.as_str())
	}
}

impl<H> From<Text<'_, H>> for String {
	#[inline]
	fn from(string: Text<'_, H>) -> Self {
		string.to_string()
	}
}

impl<H> From<Text<'_, H>> for OsString {
	fn from(string: Text<'_, H>) -> Self {
		Self::from(&string[..])
	}
}

impl<H> From<Text<'_, H>> for PathBuf {
	fn from(string: Text<'_, H>) -> Self {
		Self::from(&string[..])
	}
}

#[allow(box_pointers)]
impl<H> From<Text<'_, H>> for Box<dyn Error> {
	fn from(string: Text<'_, H>) -> Self {
		Self::from(&string[..])
	}
}

#[allow(box_pointers)]
impl<H> From<Text<'_, H>> for Box<dyn Error + Send + Sync> {
	fn from(string: Text<'_, H>) -> Self {
		Self::from(&string[..])
	}
}

impl<'system, H> Extend<Text<'system, H>> for String
where
	H: 'system,
{
	fn extend<I: IntoIterator<Item = Text<'system, H>>>(&mut self, iter: I) {
		for s in iter {
			self.push_str(&s)
		}
	}
}

impl<H> Deref for Text<'_, H> {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		self.as_str()
	}
}

impl<H> Debug for Text<'_, H> {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		Debug::fmt(&self[..], fmt)
	}
}

impl<H> Display for Text<'_, H> {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self[..], fmt)
	}
}

macro_rules! impl_index {
	($ty:ty) => {
		impl<H> Index<$ty> for Text<'_, H> {
			type Output = str;

			#[inline]
			fn index(&self, index: $ty) -> &str {
				Index::index(self.as_str(), index)
			}
		}
	};
}

impl_index!(RangeFull);
impl_index!(Range<usize>);
impl_index!(RangeTo<usize>);
impl_index!(RangeFrom<usize>);
impl_index!(RangeInclusive<usize>);
impl_index!(RangeToInclusive<usize>);

macro_rules! impl_as_ref {
	($ty:ty) => {
		impl<H> AsRef<$ty> for Text<'_, H> {
			#[inline]
			fn as_ref(&self) -> &$ty {
				AsRef::as_ref(self.as_str())
			}
		}
	};
}

impl_as_ref!(str);
impl_as_ref!([u8]);
impl_as_ref!(OsStr);
impl_as_ref!(Path);

impl<H> Borrow<str> for Text<'_, H> {
	#[inline]
	fn borrow(&self) -> &str {
		self
	}
}

impl<H> PartialEq for Text<'_, H> {
	#[inline]
	fn eq(&self, other: &Self) -> bool {
		let self_system: *const _ = &self.system;
		let other_system: *const _ = &other.system;
		if self_system == other_system {
			self.id == other.id
		} else {
			self.as_str() == other.as_str()
		}
	}
}

impl<H> Eq for Text<'_, H> {}

impl<H> PartialOrd for Text<'_, H> {
	#[inline]
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		let self_system: *const _ = &self.system;
		let other_system: *const _ = &other.system;
		if self_system == other_system && self.id == other.id {
			Some(Ordering::Equal)
		} else {
			PartialOrd::partial_cmp(self.as_str(), other.as_str())
		}
	}
}

impl<H> Ord for Text<'_, H> {
	fn cmp(&self, other: &Self) -> Ordering {
		let self_system: *const _ = &self.system;
		let other_system: *const _ = &other.system;
		if self_system == other_system && self.id == other.id {
			Ordering::Equal
		} else {
			Ord::cmp(self.as_str(), other.as_str())
		}
	}
}

macro_rules! impl_cmp {
	($ty:ty) => {
		impl<H> PartialEq<$ty> for Text<'_, H> {
			#[inline]
			fn eq(&self, other: &$ty) -> bool {
				PartialEq::eq(&self[..], &other[..])
			}
		}

		impl<H> PartialEq<Text<'_, H>> for $ty {
			#[inline]
			fn eq(&self, other: &Text<'_, H>) -> bool {
				PartialEq::eq(&self[..], &other[..])
			}
		}

		impl<H> PartialOrd<$ty> for Text<'_, H> {
			#[inline]
			fn partial_cmp(&self, other: &$ty) -> Option<Ordering> {
				PartialOrd::partial_cmp(&self[..], &other[..])
			}
		}

		impl<H> PartialOrd<Text<'_, H>> for $ty {
			#[inline]
			fn partial_cmp(&self, other: &Text<'_, H>) -> Option<Ordering> {
				PartialOrd::partial_cmp(&self[..], &other[..])
			}
		}
	};
}

impl_cmp! { str }
impl_cmp! { &str }
impl_cmp! { String }
impl_cmp! { Cow<'_, str> }

#[cfg(test)]
mod test {
	#![allow(clippy::non_ascii_literal, clippy::shadow_unrelated)]

	use astral::third_party::slog;

	use crate::System;

	use super::*;

	#[cfg(target_pointer_width = "64")]
	#[test]
	fn test_size() {
		assert_eq!(std::mem::size_of::<Text<'_>>(), 16);
		assert_eq!(std::mem::size_of::<Option<Text<'_>>>(), 16);
	}

	#[cfg(target_pointer_width = "32")]
	#[test]
	fn test_size() {
		assert_eq!(std::mem::size_of::<Text<'_>>(), 8);
		assert_eq!(std::mem::size_of::<Option<Text<'_>>>(), 8);
	}

	#[test]
	fn test_from_utf8() {
		let logger = slog::Logger::root(slog::Discard, slog::o!());
		let engine = astral::Engine::new(&logger);
		let system = System::new(&engine);
		let string_subsystem = Subsystem::new(64, &system);
		let xs = b"hello";
		assert_eq!(
			Text::from_utf8(xs, &string_subsystem).unwrap(),
			Text::new("hello", &string_subsystem)
		);

		let xs = "ศไทย中华Việt Nam".as_bytes();
		assert_eq!(
			Text::from_utf8(xs, &string_subsystem).unwrap(),
			Text::new("ศไทย中华Việt Nam", &string_subsystem)
		);
	}

	#[test]
	fn test_from_utf8_lossy() {
		let logger = slog::Logger::root(slog::Discard, slog::o!());
		let engine = astral::Engine::new(&logger);
		let system = System::new(&engine);
		let string_subsystem = Subsystem::new(64, &system);
		let xs = b"hello";
		assert_eq!(Text::from_utf8_lossy(xs, &string_subsystem), "hello");

		let xs = "ศไทย中华Việt Nam".as_bytes();
		let ys = "ศไทย中华Việt Nam";
		assert_eq!(Text::from_utf8_lossy(xs, &string_subsystem), ys);

		let xs = b"Hello\xC2 There\xFF Goodbye";
		assert_eq!(
			Text::from_utf8_lossy(xs, &string_subsystem),
			Text::new("Hello\u{FFFD} There\u{FFFD} Goodbye", &string_subsystem)
		);

		let xs = b"Hello\xC0\x80 There\xE6\x83 Goodbye";
		assert_eq!(
			Text::from_utf8_lossy(xs, &string_subsystem),
			Text::new(
				"Hello\u{FFFD}\u{FFFD} There\u{FFFD} Goodbye",
				&string_subsystem
			)
		);

		let xs = b"\xF5foo\xF5\x80bar";
		assert_eq!(
			Text::from_utf8_lossy(xs, &string_subsystem),
			Text::new("\u{FFFD}foo\u{FFFD}\u{FFFD}bar", &string_subsystem)
		);

		let xs = b"\xF1foo\xF1\x80bar\xF1\x80\x80baz";
		assert_eq!(
			Text::from_utf8_lossy(xs, &string_subsystem),
			Text::new("\u{FFFD}foo\u{FFFD}bar\u{FFFD}baz", &string_subsystem)
		);

		let xs = b"\xF4foo\xF4\x80bar\xF4\xBFbaz";
		assert_eq!(
			Text::from_utf8_lossy(xs, &string_subsystem),
			Text::new(
				"\u{FFFD}foo\u{FFFD}bar\u{FFFD}\u{FFFD}baz",
				&string_subsystem
			)
		);

		let xs = b"\xF0\x80\x80\x80foo\xF0\x90\x80\x80bar";
		assert_eq!(
			Text::from_utf8_lossy(xs, &string_subsystem),
			Text::new(
				"\u{FFFD}\u{FFFD}\u{FFFD}\u{FFFD}foo\u{10000}bar",
				&string_subsystem
			)
		);

		// surrogates
		let xs = b"\xED\xA0\x80foo\xED\xBF\xBFbar";
		assert_eq!(
			Text::from_utf8_lossy(xs, &string_subsystem),
			Text::new(
				"\u{FFFD}\u{FFFD}\u{FFFD}foo\u{FFFD}\u{FFFD}\u{FFFD}bar",
				&string_subsystem
			)
		);
	}

	#[test]
	fn test_from_utf16() {
		let logger = slog::Logger::root(slog::Discard, slog::o!());
		let engine = astral::Engine::new(&logger);
		let system = System::new(&engine);
		let string_subsystem = Subsystem::new(64, &system);
		let pairs: [(Text<'_>, Vec<u16>); 5] = [(Text::new("𐍅𐌿𐌻𐍆𐌹𐌻𐌰\n", &string_subsystem),
                  vec![0xd800, 0xdf45, 0xd800, 0xdf3f, 0xd800, 0xdf3b, 0xd800, 0xdf46, 0xd800,
                       0xdf39, 0xd800, 0xdf3b, 0xd800, 0xdf30, 0x000a]),

                 (Text::new("𐐒𐑉𐐮𐑀𐐲𐑋 𐐏𐐲𐑍\n", &string_subsystem),
                  vec![0xd801, 0xdc12, 0xd801, 0xdc49, 0xd801, 0xdc2e, 0xd801, 0xdc40, 0xd801,
                       0xdc32, 0xd801, 0xdc4b, 0x0020, 0xd801, 0xdc0f, 0xd801, 0xdc32, 0xd801,
                       0xdc4d, 0x000a]),

                 (Text::new("𐌀𐌖𐌋𐌄𐌑𐌉·𐌌𐌄𐌕𐌄𐌋𐌉𐌑\n", &string_subsystem),
                  vec![0xd800, 0xdf00, 0xd800, 0xdf16, 0xd800, 0xdf0b, 0xd800, 0xdf04, 0xd800,
                       0xdf11, 0xd800, 0xdf09, 0x00b7, 0xd800, 0xdf0c, 0xd800, 0xdf04, 0xd800,
                       0xdf15, 0xd800, 0xdf04, 0xd800, 0xdf0b, 0xd800, 0xdf09, 0xd800, 0xdf11,
                       0x000a]),

                 (Text::new("𐒋𐒘𐒈𐒑𐒛𐒒 𐒕𐒓 𐒈𐒚𐒍 𐒏𐒜𐒒𐒖𐒆 𐒕𐒆\n", &string_subsystem),
                  vec![0xd801, 0xdc8b, 0xd801, 0xdc98, 0xd801, 0xdc88, 0xd801, 0xdc91, 0xd801,
                       0xdc9b, 0xd801, 0xdc92, 0x0020, 0xd801, 0xdc95, 0xd801, 0xdc93, 0x0020,
                       0xd801, 0xdc88, 0xd801, 0xdc9a, 0xd801, 0xdc8d, 0x0020, 0xd801, 0xdc8f,
                       0xd801, 0xdc9c, 0xd801, 0xdc92, 0xd801, 0xdc96, 0xd801, 0xdc86, 0x0020,
                       0xd801, 0xdc95, 0xd801, 0xdc86, 0x000a]),
                 (Text::new("\u{20000}", &string_subsystem), vec![0xD840, 0xDC00])];

		for p in &pairs {
			let (s, u) = (*p).clone();
			let s_str = s.as_str();
			let s_as_utf16 = s_str.encode_utf16().collect::<Vec<u16>>();
			let u_as_string = Text::from_utf16(&u, &string_subsystem).unwrap().as_str();

			assert!(std::char::decode_utf16(u.iter().cloned()).all(|r| r.is_ok()));
			assert_eq!(s_as_utf16, u);

			assert_eq!(u_as_string, s);
			assert_eq!(Text::from_utf16_lossy(&u, &string_subsystem), s);

			assert_eq!(Text::from_utf16(&s_as_utf16, &string_subsystem).unwrap(), s);
			assert_eq!(u_as_string.encode_utf16().collect::<Vec<u16>>(), u);
		}
	}

	#[test]
	fn test_utf16_invalid() {
		let logger = slog::Logger::root(slog::Discard, slog::o!());
		let engine = astral::Engine::new(&logger);
		let system = System::new(&engine);
		let string_subsystem = Subsystem::new(64, &system);

		// completely positive cases tested above.
		// lead + eof
		assert!(Text::from_utf16(&[0xD800], &string_subsystem).is_err());
		// lead + lead
		assert!(Text::from_utf16(&[0xD800, 0xD800], &string_subsystem).is_err());

		// isolated trail
		assert!(Text::from_utf16(&[0x0061, 0xDC00], &string_subsystem).is_err());

		// general
		assert!(Text::from_utf16(&[0xD800, 0xd801, 0xdc8b, 0xD800], &string_subsystem).is_err());
	}

	#[test]
	fn test_from_utf16_lossy() {
		let logger = slog::Logger::root(slog::Discard, slog::o!());
		let engine = astral::Engine::new(&logger);
		let system = System::new(&engine);
		let string_subsystem = Subsystem::new(64, &system);

		// completely positive cases tested above.
		// lead + eof
		assert_eq!(
			Text::from_utf16_lossy(&[0xD800], &string_subsystem),
			Text::new("\u{FFFD}", &string_subsystem)
		);
		// lead + lead
		assert_eq!(
			Text::from_utf16_lossy(&[0xD800, 0xD800], &string_subsystem),
			Text::new("\u{FFFD}\u{FFFD}", &string_subsystem)
		);

		// isolated trail
		assert_eq!(
			Text::from_utf16_lossy(&[0x0061, 0xDC00], &string_subsystem),
			Text::new("a\u{FFFD}", &string_subsystem)
		);

		// general
		assert_eq!(
			Text::from_utf16_lossy(&[0xD800, 0xd801, 0xdc8b, 0xD800], &string_subsystem),
			Text::new("\u{FFFD}𐒋\u{FFFD}", &string_subsystem)
		);
	}

	#[allow(clippy::string_extend_chars)]
	// Returning `mut` is allowed because of `UnsafeCell`
	#[test]
	fn test_from_iterator() {
		let logger = slog::Logger::root(slog::Discard, slog::o!());
		let engine = astral::Engine::new(&logger);
		let system = System::new(&engine);
		let string_subsystem = Subsystem::new(64, &system);

		let s = Text::new("ศไทย中华Việt Nam", &string_subsystem);
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
