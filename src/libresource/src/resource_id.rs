// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use astral_core::string::Name;

use crate::assets::{Location, NamespaceId};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId {
	namespace_id: Option<NamespaceId>,
	name: Name,
}

impl ResourceId {
	pub(in crate) fn from_name(name: Name) -> Self {
		Self {
			namespace_id: None,
			name,
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

	pub fn name(self) -> Name {
		self.name
	}
}
