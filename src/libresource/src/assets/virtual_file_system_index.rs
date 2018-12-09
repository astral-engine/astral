// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::fmt::{self, Debug, Formatter};

use astral_core::collections::slot_map::Key;

/// An index to access a [`VirtualFileSystem`] inside of a [`Namespace`].
///
/// [`VirtualFileSystem`]: trait.VirtualFileSystem.html
/// [`Namespace`]: struct.Namespace.html
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct VirtualFileSystemIndex(Key<u32>);

impl VirtualFileSystemIndex {
	pub(in crate) fn new(key: Key<u32>) -> Self {
		VirtualFileSystemIndex(key)
	}

	pub(in crate) fn key(self) -> Key<u32> {
		self.0
	}
}

impl Debug for VirtualFileSystemIndex {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		Debug::fmt(&self.0, fmt)
	}
}
