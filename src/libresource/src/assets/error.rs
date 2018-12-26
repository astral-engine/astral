// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::fmt::{self, Display, Formatter};

use astral_core::error;

/// A specialized Error type with an asset [`ErrorKind`].
///
/// [`ErrorKind`]: enum.ErrorKind.html
pub type Error = error::Error<ErrorKind>;

/// The type returned from asset methods.
pub type Result<T> = error::Result<T, ErrorKind>;

/// A list specifying general categories of assets error.
///
/// It is used with the [`Error`] type.
///
/// [`Error`]: ../../core/error/struct.Error.html
// ToDo(#5): Use `non_exhaustive`
#[cfg_attr(unstable, non_exhaustive)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorKind {
	/// An I/O operation failed.
	Io,
	/// The passed index is not valid.
	InvalidIndex,
	/// Any assets error not part of this list.
	Other,
	#[doc(hidden)]
	#[allow(non_camel_case_types)]
	#[cfg(not(unstable))]
	__NON_EXHAUSTIVE,
}

impl Display for ErrorKind {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		match self {
			ErrorKind::Io => write!(fmt, "io error"),
			ErrorKind::InvalidIndex => write!(fmt, "invalid index error"),
			ErrorKind::Other => write!(fmt, "other assets error"),
			#[cfg(not(unstable))]
			ErrorKind::__NON_EXHAUSTIVE => unreachable!(),
		}
	}
}
