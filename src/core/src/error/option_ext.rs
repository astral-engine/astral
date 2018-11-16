// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::error;

use super::Error;

/// Extension methods for [`Option`].
///
/// [`Option`]: https://doc.rust-lang.org/std/option/enum.Option.html
pub trait OptionExt<T> {
	/// Transforms the `Option<T>` into a [`Result<T, Error<Kind>>`],
	/// mapping [`Some(v)`] to [`Ok(v)`] and [`None`] to
	/// [`Err(Error::new(kind, context))`].
	///
	/// [`Option<T>`]: https://doc.rust-lang.org/std/option/enum.Option.html
	/// [`Result<T, Error<Kind>>`]: https://doc.rust-lang.org/std/result/enum.Result.html
	/// [`Ok(v)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
	/// [`Err(Error::new(kind, context))`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err
	/// [`None`]: https://doc.rust-lang.org/std/option#variant.None
	/// [`Some(v)`]: https://doc.rust-lang.org/std/option#variant.Some
	///
	/// # Example
	///
	/// ```rust
	/// # extern crate astral;
	/// use std::{error, fmt};
	///
	/// use astral::core::error::OptionExt;
	///
	/// #[derive(Debug, PartialEq)]
	/// enum MyErrorKind {
	///     Variant,
	/// }
	///
	/// let option: Option<u32> = None;
	/// let x = option.ok_or_error(MyErrorKind::Variant, "oh no!").unwrap_err();
	///
	/// assert_eq!(x.kind(), &MyErrorKind::Variant);
	/// ```
	fn ok_or_error<Kind, Context>(
		self,
		kind: Kind,
		context: Context,
	) -> Result<T, Error<Kind>>
	where
		Context: Into<Box<dyn error::Error + Send + Sync>>;

	/// Transforms the `Option<T>` into a [`Result<T, Error<Kind>>`],
	/// mapping [`Some(v)`] to [`Ok(v)`] and [`None`] to
	/// [`Err(Error::new(kind, context))`] by applying the provided closure
	/// `FnOnce() -> impl Into<Box<dyn error::Error + Send + Sync>>`.
	///
	/// [`Option<T>`]: https://doc.rust-lang.org/std/option/enum.Option.html
	/// [`Result<T, Error<Kind>>`]: https://doc.rust-lang.org/std/result/enum.Result.html
	/// [`Ok(v)`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Ok
	/// [`Err(Error::new(kind, context))`]: https://doc.rust-lang.org/std/result/enum.Result.html#variant.Err
	/// [`None`]: https://doc.rust-lang.org/std/option#variant.None
	/// [`Some(v)`]: https://doc.rust-lang.org/std/option#variant.Some
	///
	/// # Example
	///
	/// ```rust
	/// # extern crate astral;
	/// use std::{error, fmt};
	///
	/// use astral::core::error::OptionExt;
	///
	/// #[derive(Debug, PartialEq)]
	/// enum MyErrorKind {
	///     Variant,
	/// }
	///
	/// let option: Option<u32> = None;
	/// let x = option.ok_or_error_with(MyErrorKind::Variant, || "oh no!").unwrap_err();
	///
	/// assert_eq!(x.kind(), &MyErrorKind::Variant);
	/// ```
	fn ok_or_error_with<Kind, Context, F>(
		self,
		kind: Kind,
		context: F,
	) -> Result<T, Error<Kind>>
	where
		Context: Into<Box<dyn error::Error + Send + Sync>>,
		F: FnOnce() -> Context;
}

impl<T> OptionExt<T> for Option<T> {
	fn ok_or_error<Kind, Context>(
		self,
		kind: Kind,
		context: Context,
	) -> Result<T, Error<Kind>>
	where
		Context: Into<Box<dyn error::Error + Send + Sync>>,
	{
		self.ok_or_else(|| Error::new(kind, context))
	}
	fn ok_or_error_with<Kind, Context, F>(
		self,
		kind: Kind,
		context: F,
	) -> Result<T, Error<Kind>>
	where
		Context: Into<Box<dyn error::Error + Send + Sync>>,
		F: FnOnce() -> Context,
	{
		self.ok_or_else(|| Error::new(kind, context()))
	}
}
