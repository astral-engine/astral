// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::fmt::{self, Debug, Formatter};

use astral_core::collections::slot_map::Key;

/// An index to access a [`Namespace`] inside of a [`Catalog`].
///
/// [`Namespace`]: struct.Namespace.html
/// [`Catalog`]: trait.Catalog.html
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct NamespaceId(Key<u16>);

impl NamespaceId {
	pub(in crate) fn new(key: Key<u16>) -> Self {
		NamespaceId(key)
	}

	pub(in crate) fn key(self) -> Key<u16> {
		self.0
	}
}

impl Debug for NamespaceId {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		Debug::fmt(&self.0, fmt)
	}
}
