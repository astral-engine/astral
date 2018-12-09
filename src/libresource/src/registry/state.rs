// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

/// The current state of a resource
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum State {
	/// The resource is not known to the registry.
	Unknown,
	/// The registry knows how to load the resource.
	Declared,
	/// The resource is loading.
	Loading,
	/// The resource is loaded and ready to be used.
	Loaded,
	/// The resource is loaded, but an error occurred during loading.
	LoadedWithError,
}
