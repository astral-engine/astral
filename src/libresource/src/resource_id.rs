// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::hash::BuildHasherDefault;

use astral_core::{hash::Murmur3, string::Name};

use crate::assets::{Location, NamespaceId};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId<'string, H = BuildHasherDefault<Murmur3>> {
	namespace_id: Option<NamespaceId>,
	name: Name<'string, H>,
}

impl<'string> ResourceId<'string> {
	pub(in crate) fn from_name<N>(name: N) -> Self
	where
		N: Into<Name<'string>>,
	{
		Self {
			namespace_id: None,
			name: name.into(),
		}
	}

	pub(in crate) fn from_location(location: Location) -> Self {
		Self {
			namespace_id: Some(location.namespace_id),
			name: location.name,
		}
	}

	pub fn location(self) -> Option<Location> {
		self.namespace_id
			.map(|namespace_id| Location::new(namespace_id, self.name))
	}

	pub fn name(self) -> Name<'string> {
		self.name
	}
}
