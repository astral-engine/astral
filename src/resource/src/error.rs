// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::fmt::{self, Display, Formatter};

use astral_core::error;

pub type Error = error::Error<ErrorKind>;
pub type Result<T> = error::Result<T, ErrorKind>;

/// A list specifying general categories of resource error.
///
/// It is used with the [`Error`] type.
///
/// [`Error`]: ../core/error/struct.Error.html
#[cfg_attr(unstable, non_exhaustive)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorKind {
	/// A resource could not be loaded.
	Loading,
	#[doc(hidden)]
	#[allow(non_camel_case_types)]
	#[cfg(not(unstable))]
	__NON_EXHAUSTIVE,
}

impl Display for ErrorKind {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		match self {
			ErrorKind::Loading => write!(fmt, "loading error"),
			#[cfg(not(unstable))]
			ErrorKind::__NON_EXHAUSTIVE => unreachable!(),
		}
	}
}
