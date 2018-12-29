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

/// Extension methods for [`Result`].
pub trait ResultExt<T, E> {
	/// Associates the error with an error kind.
	///
	/// # Example
	///
	/// ```
	/// use std::{error, fmt};
	///
	/// use astral_error::ResultExt;
	///
	/// #[derive(Debug)]
	/// struct CustomError;
	///
	/// impl fmt::Display for CustomError {
	///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	///         fmt::Debug::fmt(self, f)
	///     }
	/// }
	///
	/// impl error::Error for CustomError {}
	///
	/// #[derive(Debug, PartialEq)]
	/// enum MyErrorKind {
	///     Variant,
	/// }
	///
	/// let x = (|| -> Result<(), CustomError> {
	///     Err(CustomError)?
	/// })().context(MyErrorKind::Variant).unwrap_err();
	///
	/// assert_eq!(x.kind(), &MyErrorKind::Variant);
	/// ```
	fn context<Kind>(self, kind: Kind) -> Result<T, Error<Kind>>;

	/// Creates a new [`Error`], associates it with an error kind and sets the
	/// old error as source.
	///
	/// [`Error`]: crate::Error
	///
	/// # Example
	///
	/// ```
	/// use std::{error, fmt};
	///
	/// use astral_error::ResultExt;
	///
	/// #[derive(Debug)]
	/// struct CustomError;
	///
	/// impl fmt::Display for CustomError {
	///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	///         fmt::Debug::fmt(self, f)
	///     }
	/// }
	///
	/// impl error::Error for CustomError {}
	///
	/// #[derive(Debug)]
	/// enum MyErrorKind {
	///     Variant,
	/// }
	///
	/// impl fmt::Display for MyErrorKind {
	///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	///         fmt::Debug::fmt(self, f)
	///     }
	/// }
	///
	/// let x = (|| -> Result<(), CustomError> {
	///     Err(CustomError)?
	/// })().chain(MyErrorKind::Variant, "An error occured").unwrap_err();
	///
	/// assert_eq!(x.to_string(), "An error occured");
	/// ```
	fn chain<Kind, Source>(self, kind: Kind, source: Source) -> Result<T, Error<Kind>>
	where
		Source: Into<Box<dyn error::Error + Send + Sync>>;

	/// Creates a new [`Error`], associates it with an error kind and sets the
	/// old error as source by applying the provided closure
	/// `FnOnce() -> impl Into<Box<dyn error::Error + Send + Sync>>`.
	///
	/// [`Error`]: crate::Error
	///
	/// # Example
	///
	/// ```
	/// use std::{error, fmt};
	///
	/// use astral_error::ResultExt;
	///
	/// #[derive(Debug)]
	/// struct CustomError;
	///
	/// impl fmt::Display for CustomError {
	///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	///         fmt::Debug::fmt(self, f)
	///     }
	/// }
	///
	/// impl error::Error for CustomError {}
	///
	/// #[derive(Debug)]
	/// enum MyErrorKind {
	///     Variant,
	/// }
	///
	/// impl fmt::Display for MyErrorKind {
	///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	///         fmt::Debug::fmt(self, f)
	///     }
	/// }
	///
	/// let x = (|| -> Result<(), CustomError> {
	///     Err(CustomError)?
	/// })().chain_with(MyErrorKind::Variant, || "An error occured").unwrap_err();
	///
	/// assert_eq!(x.to_string(), "An error occured");
	/// ```
	fn chain_with<Kind, Source, F>(self, kind: Kind, source: F) -> Result<T, Error<Kind>>
	where
		Source: Into<Box<dyn error::Error + Send + Sync>>,
		F: FnOnce() -> Source;
}

#[allow(clippy::use_self)]
impl<T, E> ResultExt<T, E> for Result<T, E>
where
	E: Into<Box<dyn error::Error + Send + Sync>>,
{
	fn context<Kind>(self, kind: Kind) -> Result<T, Error<Kind>> {
		self.map_err(|error| Error::new(kind, error))
	}

	fn chain<Kind, Source>(self, kind: Kind, source: Source) -> Result<T, Error<Kind>>
	where
		Source: Into<Box<dyn error::Error + Send + Sync>>,
	{
		self.map_err(|s| Error::chained(kind, source.into(), s))
	}

	fn chain_with<Kind, Source, F>(self, kind: Kind, source: F) -> Result<T, Error<Kind>>
	where
		Source: Into<Box<dyn error::Error + Send + Sync>>,
		F: FnOnce() -> Source,
	{
		self.map_err(|s| Error::chained(kind, source(), s.into()))
	}
}
