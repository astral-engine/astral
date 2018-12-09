// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::{
	error,
	fmt::{self, Debug, Display, Formatter},
};

use super::{Chained, Custom};

#[allow(variant_size_differences)]
pub(super) enum Repr<Kind> {
	Simple(Kind),
	Custom(Box<Custom<Kind>>),
	Chained(Box<Chained<Kind>>),
}

impl<Kind> Repr<Kind> {
	pub(super) fn get_ref(&self) -> Option<&(dyn error::Error + Send + Sync + 'static)> {
		match self {
			Repr::Simple(..) => None,
			Repr::Custom(c) => Some(c.error.as_ref()),
			Repr::Chained(c) => Some(c.error.as_ref()),
		}
	}

	pub(super) fn get_mut(&mut self) -> Option<&mut (dyn error::Error + Send + Sync + 'static)> {
		match self {
			Repr::Simple(..) => None,
			Repr::Custom(c) => Some(c.error.as_mut()),
			Repr::Chained(c) => Some(c.error.as_mut()),
		}
	}

	pub(super) fn into_inner(self) -> Option<Box<dyn error::Error + Send + Sync>> {
		match self {
			Repr::Simple(..) => None,
			Repr::Custom(c) => Some(c.error),
			Repr::Chained(c) => Some(c.error),
		}
	}

	pub(super) fn kind(&self) -> &Kind {
		match self {
			Repr::Simple(ref kind) => kind,
			Repr::Custom(ref c) => &c.kind,
			Repr::Chained(ref c) => &c.kind,
		}
	}
}

impl<Kind> Debug for Repr<Kind>
where
	Kind: Debug,
{
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		match *self {
			Repr::Simple(ref kind) => fmt.debug_tuple("Kind").field(&kind).finish(),
			Repr::Custom(ref c) => Debug::fmt(&c, fmt),
			Repr::Chained(ref c) => Debug::fmt(&c, fmt),
		}
	}
}

impl<Kind> Display for Repr<Kind>
where
	Kind: Display,
{
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		match self {
			Repr::Simple(kind) => Display::fmt(&kind, fmt),
			Repr::Custom(ref c) => Display::fmt(&c.error, fmt),
			Repr::Chained(ref c) => Display::fmt(&c.error, fmt),
		}
	}
}

impl<Kind> error::Error for Repr<Kind>
where
	Kind: Debug + Display,
{
	fn source(&self) -> Option<&(dyn error::Error + 'static)> {
		match self {
			Repr::Simple(..) => None,
			Repr::Custom(ref c) => c.error.source(),
			Repr::Chained(ref c) => Some(c.source.as_ref()),
		}
	}
}
