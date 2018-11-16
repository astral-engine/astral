// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::result;

use super::Error;

/// A specialized [`Result`] type in the Astral Engine.
///
/// This type is broadly used across [`astral`] for any operation which may
/// produce an error.
///
/// This typedef is generally used to avoid writing out
/// [`astral::Error`] directly and is otherwise a direct mapping
/// to [`Result`].
///
/// While usual Rust style is to import types directly, aliases of [`Result`]
/// often are not, to make it easier to distinguish between them. [`Result`] is
/// generally assumed to be [`std::result::Result`][`Result`], and so users of
/// this alias will generally use `astral::Result` instead of shadowing the
/// prelude's import of [`std::result::Result`][`Result`].
///
/// [`astral`]: ../../index.html
/// [`astral::Error`]: struct.Error.html
/// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
pub type Result<T, Kind> = result::Result<T, Error<Kind>>;
