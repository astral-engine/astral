// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::hash::BuildHasherDefault;

use astral_core::{hash::Murmur3, string::Name};

use super::NamespaceId;

/// A `Location` consists of a [`NamespaceId`] and a [`Name`].
///
/// It is used to uniquely identify an asset in a [`Catalog`].
///
/// [`NamespaceId`]: struct.NamespaceId.html
/// [`Catalog`]: struct.Catalog.html
/// [`Name`]: ../../core/string/struct.Name.html
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Location<'str, H = BuildHasherDefault<Murmur3>> {
	pub namespace_id: NamespaceId,
	pub name: Name<'str, H>,
}

impl<H> Clone for Location<'_, H> {
	fn clone(&self) -> Self {
		Self {
			namespace_id: self.namespace_id,
			name: self.name,
		}
	}
}

impl<H> Copy for Location<'_, H> {}

impl<'str, H> Location<'str, H> {
	/// Construct a `Location` from a [`NamespaceId`] and a [`Name`].
	///
	/// [`NamespaceId`]: struct.NamespaceId.html
	/// [`Name`]: ../../core/string/struct.Name.html
	pub fn new(namespace_id: NamespaceId, name: Name<'str, H>) -> Self {
		Self { namespace_id, name }
	}

	/// Construct a `Location` from a [`NamespaceId`] and a string, which can be converted
	/// into a name [`Name`].
	///
	/// [`NamespaceId`]: struct.NamespaceId.html
	/// [`Name`]: ../../core/string/struct.Name.html
	pub fn from_string<S>(namespace_id: NamespaceId, name: S) -> Self
	where
		S: Into<Name<'str, H>>,
	{
		Self {
			namespace_id,
			name: name.into(),
		}
	}
}
