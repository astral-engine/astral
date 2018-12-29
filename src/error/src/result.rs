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
/// While usual Rust style is to import types directly, aliases of `Result`
/// often are not, to make it easier to distinguish between them. `Result` is
/// generally assumed to be [`std::result::Result`], and so users of
/// this alias will generally use `astral::Result` instead of shadowing the
/// prelude's import of `std::result::Result`.
///
/// [`astral`]: ../../index.html
/// [`astral::Error`]: struct.Error.html
pub type Result<T, Kind> = result::Result<T, Error<Kind>>;
