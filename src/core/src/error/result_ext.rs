// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::error;

use super::Error;

/// Extension methods for [`Result`].
pub trait ResultExt<T, E> {
	/// Associates the error with an error kind.
	///
	/// # Example
	///
	/// ```rust
	/// use std::{error, fmt};
	///
	/// use astral::core::error::ResultExt;
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
	/// [`Error`]: error::Error
	///
	/// # Example
	///
	/// ```rust
	/// use std::{error, fmt};
	///
	/// use astral::core::error::ResultExt;
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
	fn chain<Kind, Context>(
		self,
		kind: Kind,
		context: Context,
	) -> Result<T, Error<Kind>>
	where
		Context: Into<Box<dyn error::Error + Send + Sync>>;

	/// Creates a new [`Error`], associates it with an error kind and sets the
	/// old error as source by applying the provided closure
	/// `FnOnce() -> impl Into<Box<dyn error::Error + Send + Sync>>`.
	///
	/// [`Error`]: error::Error
	///
	/// # Example
	///
	/// ```rust
	/// use std::{error, fmt};
	///
	/// use astral::core::error::ResultExt;
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
	fn chain_with<Kind, Context, F>(
		self,
		kind: Kind,
		context: F,
	) -> Result<T, Error<Kind>>
	where
		Context: Into<Box<dyn error::Error + Send + Sync>>,
		F: FnOnce() -> Context;
}

#[allow(clippy::use_self)]
impl<T, E> ResultExt<T, E> for Result<T, E>
where
	E: Into<Box<dyn error::Error + Send + Sync>>,
{
	fn context<Kind>(self, kind: Kind) -> Result<T, Error<Kind>> {
		self.map_err(|error| Error::new(kind, error))
	}

	fn chain<Kind, Context>(
		self,
		kind: Kind,
		context: Context,
	) -> Result<T, Error<Kind>>
	where
		Context: Into<Box<dyn error::Error + Send + Sync>>,
	{
		self.map_err(|source| Error::chained(kind, context.into(), source))
	}

	fn chain_with<Kind, Context, F>(
		self,
		kind: Kind,
		context: F,
	) -> Result<T, Error<Kind>>
	where
		Context: Into<Box<dyn error::Error + Send + Sync>>,
		F: FnOnce() -> Context,
	{
		self.map_err(|source| Error::chained(kind, context(), source.into()))
	}
}
