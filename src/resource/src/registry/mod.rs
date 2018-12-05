// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

mod closures;
mod state;

pub use self::state::State;

use std::{
	collections::HashMap,
	error,
	fmt::{self, Debug, Formatter},
	hash::BuildHasherDefault,
	io::Read,
	mem, result,
	sync::Arc,
};

use astral_core::{
	error::{OptionExt, ResultExt},
	hash::NopHasher,
	string::Name,
};

use crate::{
	assets::{Catalog, Location},
	ErrorKind, Resource, ResourceId, Result,
};

use self::closures::{AssetLoader, Closures, ResourceLoader};

pub struct Loader<R, P> {
	catalog: Option<Arc<Catalog<'static>>>,
	default_resource_loader: ResourceLoader<R, P>,
	default_asset_loader: AssetLoader<R, P>,
	declarations: HashMap<
		ResourceId,
		Option<Closures<R, P>>,
		BuildHasherDefault<NopHasher>,
	>,
}

impl<R, P> Loader<R, P>
where
	R: Resource + 'static,
{
	pub fn new<F1, F2>(resource_loader: F1, asset_loader: F2) -> Self
	where
		F1: Fn(P) -> result::Result<R, Box<dyn error::Error + Send + Sync>>
			+ Send
			+ Sync
			+ 'static,
		F2: Fn(
				P,
				&mut (dyn Read),
			) -> result::Result<R, Box<dyn error::Error + Send + Sync>>
			+ Send
			+ Sync
			+ 'static,
	{
		Self {
			catalog: None,
			default_resource_loader: Arc::new(move |parameters| {
				resource_loader(parameters).context(ErrorKind::Loading)
			}),
			default_asset_loader: Arc::new(move |parameters, read| {
				asset_loader(parameters, read).context(ErrorKind::Loading)
			}),
			declarations: HashMap::default(),
		}
	}

	pub fn set_catalog<C>(
		&mut self,
		catalog: C,
	) -> Option<Arc<Catalog<'static>>>
	where
		C: Into<Arc<Catalog<'static>>>,
	{
		mem::replace(&mut self.catalog, Some(catalog.into()))
	}

	pub fn catalog(&self) -> Option<Arc<Catalog<'static>>> {
		self.catalog.as_ref().cloned()
	}

	pub fn declare_resource(&mut self, name: Name) -> ResourceId {
		let resource_id = ResourceId::from_name(name);
		self.declarations.insert(resource_id, None);
		resource_id
	}

	pub fn declare_resource_with_loader<F>(
		&mut self,
		name: Name,
		loader: F,
	) -> ResourceId
	where
		F: Fn(P) -> result::Result<R, Box<dyn error::Error + Send + Sync>>
			+ Send
			+ Sync
			+ 'static,
	{
		let resource_id = ResourceId::from_name(name);
		self.declarations
			.insert(resource_id, Some(Closures::new_resource(loader)));
		resource_id
	}

	pub fn declare_asset(&mut self, location: Location) -> ResourceId {
		let resource_id = ResourceId::from_location(location);
		self.declarations.insert(resource_id, None);
		resource_id
	}

	pub fn declare_asset_with_loader<F>(
		&mut self,
		location: Location,
		loader: F,
	) -> ResourceId
	where
		F: Fn(
				P,
				&mut (dyn Read),
			) -> result::Result<R, Box<dyn error::Error + Send + Sync>>
			+ Send
			+ Sync
			+ 'static,
	{
		let resource_id = ResourceId::from_location(location);
		self.declarations
			.insert(resource_id, Some(Closures::new_asset(loader)));
		resource_id
	}

	fn loader_catalog(
		&self,
		resource_id: ResourceId,
	) -> Result<(Closures<R, P>, Arc<Catalog<'static>>)> {
		let declaration = self
			.declarations
			.get(&resource_id)
			.ok_or_error(ErrorKind::Loading, "asset was not declared")?
			.clone();

		let loader = if let Some(loader) = declaration {
			loader.clone()
		} else if resource_id.location().is_some() {
			Closures::Asset(self.default_asset_loader.clone())
		} else {
			Closures::Resource(self.default_resource_loader.clone())
		};

		let catalog = self
			.catalog()
			.ok_or_error(ErrorKind::Loading, "no catalog set")?
			.clone();

		Ok((loader, catalog))
	}

	fn load_impl(
		resource_id: ResourceId,
		catalog: &Catalog<'_>,
		loader: Closures<R, P>,
		parameters: P,
	) -> Result<R> {
		match loader {
			Closures::Asset(loader) => {
				let mut read = catalog
					.open(resource_id.location().unwrap())
					.ok_or_error(
						ErrorKind::Loading,
						"location could not be found in catalog",
					)?
					.context(ErrorKind::Loading)?;
				loader(parameters, &mut read)
			}
			Closures::Resource(loader) => loader(parameters),
		}
	}

	pub fn load(&self, resource_id: ResourceId, parameters: P) -> Result<R> {
		let (loader, catalog) = self.loader_catalog(resource_id)?;
		Self::load_impl(resource_id, catalog.as_ref(), loader, parameters)
	}

	// pub fn load_lazy(
	// 	&self,
	// 	resource_id: ResourceId,
	// 	parameters: P,
	// ) -> Result<impl Future<Output = Result<R>>>
	// {
	// 	let (loader, catalog) = self.loader_catalog(resource_id)?;
	// 	Ok(future::lazy(move |_| {
	// 		Self::load_impl(resource_id, catalog.as_ref(), loader, parameters)
	// 	}))
	// }

	pub fn clear(&mut self) {
		self.declarations.clear();
	}
}

impl<R, P> Debug for Loader<R, P> {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		fmt.debug_struct("Registry")
			.field("catalog", &self.catalog)
			.finish()
	}
}
