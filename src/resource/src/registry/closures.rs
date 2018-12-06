// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::{error, io::Read, result, sync::Arc};

use astral_core::error::ResultExt;

use crate::{ErrorKind, Result};

pub type ResourceLoader<R, P> = Arc<dyn Fn(P) -> Result<R> + Send + Sync + 'static>;
pub type AssetLoader<R, P> = Arc<dyn Fn(P, &mut (dyn Read)) -> Result<R> + Send + Sync + 'static>;

pub enum Closures<R, P> {
	Resource(ResourceLoader<R, P>),
	Asset(AssetLoader<R, P>),
}

impl<R, P> Clone for Closures<R, P> {
	fn clone(&self) -> Self {
		match self {
			Closures::Resource(loader) => Closures::Resource(loader.clone()),
			Closures::Asset(loader) => Closures::Asset(loader.clone()),
		}
	}
}

impl<R, P> Closures<R, P> {
	pub fn new_resource<F>(loader: F) -> Self
	where
		F: Fn(P) -> result::Result<R, Box<dyn error::Error + Send + Sync>> + Send + Sync + 'static,
	{
		Closures::Resource(Arc::new(move |parameters| {
			loader(parameters).chain(ErrorKind::Loading, "could not load asset")
		}))
	}

	pub fn new_asset<F>(loader: F) -> Self
	where
		F: Fn(P, &mut (dyn Read)) -> result::Result<R, Box<dyn error::Error + Send + Sync>>
			+ Send
			+ Sync
			+ 'static,
	{
		Closures::Asset(Arc::new(move |parameters, read| {
			loader(parameters, read).chain(ErrorKind::Loading, "could not load asset")
		}))
	}
}
