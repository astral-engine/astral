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

//! Traits and structures for working with Errors in the Astral Engine.

#![doc(
	html_no_source,
	html_logo_url = "https://astral-engine.github.io/docs/logo_astral.svg",
	html_favicon_url = "https://astral-engine.github.io/docs/logo.svg",
	test(attr(
		deny(
			future_incompatible,
			nonstandard_style,
			rust_2018_compatibility,
			rust_2018_idioms,
			unused,
			macro_use_extern_crate,
			trivial_casts,
			trivial_numeric_casts,
			unused_import_braces,
			unused_lifetimes,
			unused_qualifications,
			variant_size_differences,
		),
		allow(unused_extern_crates)
	))
)]
#![warn(
	future_incompatible,
	nonstandard_style,
	rust_2018_compatibility,
	rust_2018_idioms,
	unused,
	macro_use_extern_crate,
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs,
	// missing_doc_code_examples,
	single_use_lifetimes,
	trivial_casts,
	trivial_numeric_casts,
	unreachable_pub,
	unused_import_braces,
	unused_lifetimes,
	unused_qualifications,
	unused_results,
	variant_size_differences,
	clippy::pedantic
)]

mod chained;
mod custom;
mod option_ext;
mod repr;
mod result;
mod result_ext;

pub use self::{option_ext::OptionExt, result::Result, result_ext::ResultExt};

use std::{
	error,
	fmt::{self, Debug, Display, Formatter},
};

use self::{chained::Chained, custom::Custom, repr::Repr};

/// The generic error type for the Astral engine.
///
/// `Error` can be created with crafted error messages and a particular value of
/// `Kind` and optionally with a arbitrary error payload.
///
/// It is useful but not necessary, that `Kind` implements [`Debug`] and
/// [`Display`] so [`std::error::Error`] is implemented.
///
/// [`Debug`]: std::fmt::Debug
/// [`Display`]: std::fmt::Display
/// [`Error`]: std::error::Error
///
/// # Example
///
/// ```
/// use std::fmt::{self, Debug, Display, Formatter};
/// use std::error::Error as StdError;
///
/// use astral_error::Error;
///
/// #[derive(Debug, PartialEq)]
/// enum MyErrorKind {
///     Variant,
/// }
///
/// impl Display for MyErrorKind {
///     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
///         Debug::fmt(self, f)
///     }
/// }
///
/// let my_error = Error::new(MyErrorKind::Variant, "oh no!");
///
/// let my_error2 = Error::new(MyErrorKind::Variant, my_error);
///
/// assert_eq!(*my_error2.kind(), MyErrorKind::Variant);
/// assert!(my_error2.source().is_none());
/// ```
pub struct Error<Kind> {
	repr: Repr<Kind>,
}

impl<Kind> Error<Kind> {
	/// Creates a new error from a known kind of error as well as an arbitrary
	/// error payload. The `error` argument is an arbitrary payload which will
	/// be contained in this `Error`. The resulting error don't have a source
	/// error returned by [`Error::source`].
	///
	/// [`Error::source`]: std::error::Error::source
	///
	/// # Example
	///
	/// ```rust
	/// # use std::fmt::{self, Debug, Display, Formatter};
	/// # #[derive(Debug, PartialEq)] enum MyErrorKind { Variant }
	/// # impl Display for MyErrorKind {
	/// #     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
	/// #         Debug::fmt(self, f)
	/// #     }
	/// # }
	/// use std::error::Error as StdError;
	///
	/// use astral_error::Error;
	///
	/// let my_error = Error::new(MyErrorKind::Variant, "oh no!");
	///
	/// let my_error2 = Error::new(MyErrorKind::Variant, my_error);
	///
	/// assert!(my_error2.source().is_none());
	/// ```
	pub fn new<E>(kind: Kind, error: E) -> Self
	where
		E: Into<Box<dyn error::Error + Send + Sync>>,
	{
		Self {
			repr: Repr::Custom(Box::new(Custom {
				kind,
				error: error.into(),
			})),
		}
	}

	/// Creates a new error from a known kind of error as well as an arbitrary
	/// error payload and keeps another payload as source error.
	///
	/// The `error` argument is an arbitrary payload which will be contained in
	/// this `Error`. The `source` argument is an error, which will be returned
	/// by [`Error::source`]
	///
	/// [`Error::source`]: std::error::Error::source
	///
	/// # Example
	///
	/// ```
	/// # use std::fmt::{self, Debug, Display, Formatter};
	/// # #[derive(Debug)] enum MyErrorKind { Variant }
	/// # impl Display for MyErrorKind {
	/// #     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
	/// #         Debug::fmt(self, f)
	/// #     }
	/// # }
	/// # fn main() { test().unwrap() }
	/// # fn test() -> Option<()> {
	/// use std::error::Error as StdError;
	///
	/// use astral_error::Error;
	///
	/// let my_error = Error::new(MyErrorKind::Variant, "oh no!");
	///
	/// let my_error2 = Error::chained(MyErrorKind::Variant, "failed!", my_error);
	///
	/// assert_eq!(my_error2.source()?.to_string(), "oh no!");
	/// # Some(())
	/// # }
	/// ```
	pub fn chained<E, S>(kind: Kind, error: E, source: S) -> Self
	where
		E: Into<Box<dyn error::Error + Send + Sync>>,
		S: Into<Box<dyn error::Error + Send + Sync>>,
	{
		Self {
			repr: Repr::Chained(Box::new(Chained {
				kind,
				error: error.into(),
				source: source.into(),
			})),
		}
	}

	/// Returns a reference to the inner error wrapped by this error (if any).
	///
	/// If this `Error` was constructed via [`new`] or [`chained`] then this
	/// function will return [`Some`], otherwise it will return [`None`].
	///
	/// [`new`]: crate::Error::new
	/// [`chained`]: crate::Error::chained
	///
	/// # Examples
	///
	/// ```
	/// use astral_error::Error;
	///
	/// #[derive(Debug)]
	/// enum MyErrorKind {
	/// 	Variant,
	/// }
	///
	/// fn print_error<Kind>(err: &Error<Kind>) {
	///     if let Some(inner_err) = err.get_ref() {
	///         println!("Inner error: {:?}", inner_err);
	///     } else {
	///         println!("No inner error");
	///     }
	/// }
	///
	/// fn main() {
	///     // Will print "Inner error: Variant".
	///     print_error(&Error::new(MyErrorKind::Variant, "oh no!"));
	/// }
	/// ```
	pub fn get_ref(&self) -> Option<&(dyn error::Error + Send + Sync + 'static)> {
		self.repr.get_ref()
	}

	/// Returns a mutable reference to the inner error wrapped by this error
	/// (if any).
	///
	/// If this `Error` was constructed via [`new`] or [`chained`] then this
	/// function will return [`Some`], otherwise it will return [`None`].
	///
	/// [`new`]: crate::Error::new
	/// [`chained`]: crate::Error::chained
	///
	/// # Examples
	///
	/// ```
	/// use std::{error, fmt};
	/// use std::fmt::Display;
	///
	/// use astral_error::Error;
	///
	/// #[derive(Debug)]
	/// struct MyError {
	///     v: String,
	/// }
	///
	/// impl MyError {
	///     fn new() -> MyError {
	///         MyError {
	///             v: "oh no!".to_string()
	///         }
	///     }
	///
	///     fn change_message(&mut self, new_message: &str) {
	///         self.v = new_message.to_string();
	///     }
	/// }
	///
	/// impl error::Error for MyError {}
	///
	/// impl Display for MyError {
	///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	///         write!(f, "MyError: {}", &self.v)
	///     }
	/// }
	///
	/// fn change_error<Kind>(mut err: Error<Kind>) -> Error<Kind> {
	///     if let Some(inner_err) = err.get_mut() {
	///         inner_err.downcast_mut::<MyError>().unwrap().change_message("I've been changed!");
	///     }
	///     err
	/// }
	///
	/// #[derive(Debug)]
	/// enum MyErrorKind {
	/// 	Variant,
	/// }
	///
	/// fn print_error<Kind>(err: &Error<Kind>) {
	///     if let Some(inner_err) = err.get_ref() {
	///         println!("Inner error: {}", inner_err);
	///     } else {
	///         println!("No inner error");
	///     }
	/// }
	///
	/// fn main() {
	///     // Will print "Inner error: ...".
	///     print_error(&change_error(Error::new(MyErrorKind::Variant, MyError::new())));
	/// }
	/// ```
	pub fn get_mut(&mut self) -> Option<&mut (dyn error::Error + Send + Sync + 'static)> {
		self.repr.get_mut()
	}

	/// Consumes the `Error`, returning its inner error (if any).
	///
	/// If this `Error` was constructed via [`new`] or [`chained`] then this
	/// function will return [`Some`], otherwise it will return [`None`].
	///
	/// [`new`]: crate::Error::new
	/// [`chained`]: crate::Error::chained
	///
	/// # Example
	///
	/// ```
	/// # use std::fmt::{self, Display, Formatter};
	/// # #[derive(Debug)] enum MyErrorKind { Variant }
	/// # impl Display for MyErrorKind {
	/// #     fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
	/// #         Display::fmt(self, f)
	/// #     }
	/// # }
	/// # fn main() { test().unwrap() }
	/// # fn test() -> Option<()> {
	/// use astral_error::Error;
	///
	/// let my_error = Error::new(MyErrorKind::Variant, "oh no!");
	///
	/// let my_error2 = Error::new(MyErrorKind::Variant, my_error);
	///
	/// assert_eq!(my_error2.into_inner()?.to_string(), "oh no!");
	/// # Some(())
	/// # }
	/// ```
	#[inline]
	pub fn into_inner(self) -> Option<Box<dyn error::Error + Send + Sync>> {
		self.repr.into_inner()
	}

	/// Returns the corresponding `Kind` for this error.
	///
	/// # Example
	///
	/// ```
	/// use astral_error::Error;
	///
	/// #[derive(Debug, PartialEq)]
	/// enum MyErrorKind {
	///     Variant,
	/// }
	///
	/// let my_error = Error::new(MyErrorKind::Variant, "oh no!");
	/// assert_eq!(*my_error.kind(), MyErrorKind::Variant);
	/// ```
	#[inline]
	pub fn kind(&self) -> &Kind {
		self.repr.kind()
	}
}

impl<Kind> Debug for Error<Kind>
where
	Kind: Debug,
{
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		Debug::fmt(&self.repr, fmt)
	}
}

impl<Kind> Display for Error<Kind>
where
	Kind: Display,
{
	#[inline]
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		Display::fmt(&self.repr, fmt)
	}
}

impl<Kind> error::Error for Error<Kind>
where
	Kind: Debug + Display,
{
	#[inline]
	fn source(&self) -> Option<&(dyn error::Error + 'static)> {
		self.repr.source()
	}
}

impl<Kind> From<Kind> for Error<Kind> {
	#[inline]
	fn from(kind: Kind) -> Self {
		Self {
			repr: Repr::Simple(kind),
		}
	}
}
