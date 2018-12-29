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

use std::error;

use super::Error;

/// Extension methods for [`Option`].
pub trait OptionExt<T> {
	/// Transforms the [`Option<T>`] into a [`Result<T, Error<Kind>>`],
	/// mapping [`Some(v)`] to [`Ok(v)`] and [`None`] to
	/// [`Err(Error::new(kind, context))`].
	///
	/// [`Option<T>`]: Option
	/// [`Result<T, Error<Kind>>`]: Result
	/// [`Ok(v)`]: Ok
	/// [`Err(Error::new(kind, context))`]: Err
	/// [`Some(v)`]: Some
	///
	/// # Example
	///
	/// ```
	/// use astral_error::OptionExt;
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
	fn ok_or_error<Kind, Context>(self, kind: Kind, context: Context) -> Result<T, Error<Kind>>
	where
		Context: Into<Box<dyn error::Error + Send + Sync>>;

	/// Transforms the [`Option<T>`] into a [`Result<T, Error<Kind>>`],
	/// mapping [`Some(v)`] to [`Ok(v)`] and [`None`] to
	/// [`Err(Error::new(kind, context))`] by applying the provided closure
	/// `FnOnce() -> impl Into<Box<dyn error::Error + Send + Sync>>`.
	///
	/// [`Option<T>`]: Option
	/// [`Result<T, Error<Kind>>`]: Result
	/// [`Ok(v)`]: Ok
	/// [`Err(Error::new(kind, context))`]: Err
	/// [`Some(v)`]: Some
	///
	/// # Example
	///
	/// ```
	/// use astral_error::OptionExt;
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
	fn ok_or_error_with<Kind, Context, F>(self, kind: Kind, context: F) -> Result<T, Error<Kind>>
	where
		Context: Into<Box<dyn error::Error + Send + Sync>>,
		F: FnOnce() -> Context;
}

impl<T> OptionExt<T> for Option<T> {
	fn ok_or_error<Kind, Context>(self, kind: Kind, context: Context) -> Result<T, Error<Kind>>
	where
		Context: Into<Box<dyn error::Error + Send + Sync>>,
	{
		self.ok_or_else(|| Error::new(kind, context))
	}

	fn ok_or_error_with<Kind, Context, F>(self, kind: Kind, context: F) -> Result<T, Error<Kind>>
	where
		Context: Into<Box<dyn error::Error + Send + Sync>>,
		F: FnOnce() -> Context,
	{
		self.ok_or_else(|| Error::new(kind, context()))
	}
}
