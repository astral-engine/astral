// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::{
	fmt::{self, Debug, Formatter},
	io::{Read, Write},
	ops::{Index, IndexMut},
	time::SystemTime,
};

use astral_core::collections::SparseSlotMap;

use super::{Location, Namespace, NamespaceId, Result, VirtualFileSystemIndex};

/// A collection of [`Namespace`]s.
///
/// Each [`Namespace`] is associated with a [`NamespaceId`] for accessing
/// the [`Namespace`] at a later point.
///
/// A [`VirtualFileSystem`] can be used directly with a [`Location`].
///
/// [`Namespace`]: struct.Namespace.html
/// [`NamespaceId`]: struct.NamespaceId.html
/// [`VirtualFileSystem`]: trait.VirtualFileSystem.html
/// [`Location`]: struct.Location.html
#[derive(Default)]
pub struct Catalog<'loc> {
	namespaces: SparseSlotMap<Namespace<'loc>, u16>,
}

impl<'loc> Catalog<'loc> {
	/// Construct a new empty `Catalog`.
	///
	/// # Example
	///
	/// ```
	/// # extern crate astral;
	/// use astral::resource::assets::Catalog;
	///
	/// let catalog = Catalog::new();
	/// assert!(catalog.is_empty());
	/// ```
	pub fn new() -> Self {
		Self::default()
	}

	/// Construct a new, empty `Catalog` with the specified capacity.
	///
	/// The `Catalog` will be able to hold exactly `capacity` [`Namespace`]s
	/// without reallocating.
	/// If `capacity` is 0, the `Catalog` will not allocate.
	///
	/// [`Namespace`]: struct.Namespace.html
	///
	/// # Example
	///
	/// ```
	/// # extern crate astral;
	/// use astral::resource::assets::{Catalog, Namespace};
	///
	/// let mut catalog = Catalog::with_capacity(1);
	///
	/// assert_eq!(catalog.len(), 0);
	///
	/// // This can be done without reallocating...
	/// catalog.add_namespace(Namespace::new());
	///
	/// // ...but this may make the catalog reallocate
	/// catalog.add_namespace(Namespace::new());
	/// ```
	pub fn with_capacity(capacity: usize) -> Self {
		Self {
			namespaces: SparseSlotMap::with_capacity(capacity),
		}
	}

	/// Returns the number of [`Namespace`]s in the `Catalog`.
	///
	/// [`Namespace`]: struct.Namespace.html
	///
	/// # Example
	///
	/// ```
	/// # extern crate astral;
	/// use astral::resource::assets::{Catalog, Namespace};
	///
	/// let mut catalog = Catalog::new();
	///
	/// assert_eq!(catalog.len(), 0);
	/// catalog.add_namespace(Namespace::new());
	/// assert_eq!(catalog.len(), 1);
	/// ```
	pub fn len(&self) -> usize {
		self.namespaces.len() as usize
	}

	/// Returns `true` if the `Catalog` contains no [`Namespace`]s.
	///
	/// [`Namespace`]: struct.Namespace.html
	///
	/// # Example
	///
	/// ```
	/// # extern crate astral;
	/// use astral::resource::assets::{Catalog, Namespace};
	///
	/// let mut catalog = Catalog::new();
	///
	/// assert!(catalog.is_empty());
	/// catalog.add_namespace(Namespace::new());
	/// assert!(!catalog.is_empty());
	/// ```
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Adds a new [`Namespace`] and returns its [`NamespaceId`]
	/// to query it at a later time.
	///
	/// [`Namespace`]: struct.Namespace.html
	/// [`NamespaceId`]: struct.NamespaceId.html
	///
	/// # Example
	///
	/// ```
	/// # extern crate astral;
	/// use astral::resource::assets::{Catalog, Namespace};
	///
	/// let mut catalog = Catalog::new();
	///
	/// assert_eq!(catalog.len(), 0);
	/// let namespace_id = catalog.add_namespace(Namespace::new());
	/// assert_eq!(catalog.len(), 1);
	/// ```
	pub fn add_namespace(&mut self, namespace: Namespace<'loc>) -> NamespaceId {
		NamespaceId::new(self.namespaces.insert(namespace))
	}

	/// Returns the [`Namespace`] as reference of the given [`NamespaceId`] if any or [`None`].
	///
	/// [`None`]: https://doc.rust-lang.org/std/option/enum.Option.html#variant.None
	/// [`Namespace`]: struct.Namespace.html
	/// [`NamespaceId`]: struct.NamespaceId.html
	///
	/// # Example
	///
	/// ```
	/// # extern crate astral;
	/// use astral::resource::assets::{Catalog, Namespace};
	///
	/// let mut catalog = Catalog::new();
	///
	/// let namespace_id = catalog.add_namespace(Namespace::new());
	/// assert!(catalog.get_namespace(namespace_id).is_some());
	/// ```
	pub fn get_namespace(
		&self,
		namespace_id: NamespaceId,
	) -> Option<&Namespace<'loc>> {
		self.namespaces.get(namespace_id.key())
	}

	/// Returns the [`Namespace`] as mutable reference of the given [`NamespaceId`] if any or [`None`].
	///
	/// [`None`]: https://doc.rust-lang.org/std/option/enum.Option.html#variant.None
	/// [`Namespace`]: struct.Namespace.html
	/// [`NamespaceId`]: struct.NamespaceId.html
	///
	/// # Example
	///
	/// ```
	/// # extern crate astral;
	/// use astral::resource::assets::{Catalog, Namespace};
	///
	/// let mut catalog = Catalog::new();
	///
	/// let namespace_id = catalog.add_namespace(Namespace::new());
	/// assert!(catalog.get_namespace_mut(namespace_id).unwrap().is_empty());
	/// ```
	pub fn get_namespace_mut(
		&mut self,
		namespace_id: NamespaceId,
	) -> Option<&mut Namespace<'loc>> {
		self.namespaces.get_mut(namespace_id.key())
	}

	/// Removes the [`Namespace`] of the given [`NamespaceId`] and returns it.
	///
	/// [`Namespace`]: struct.Namespace.html
	/// [`NamespaceId`]: struct.NamespaceId.html
	///
	/// # Example
	///
	/// ```
	/// # extern crate astral;
	/// use astral::resource::assets::{Catalog, Namespace};
	///
	/// let mut catalog = Catalog::new();
	///
	/// let namespace_id = catalog.add_namespace(Namespace::new());
	/// assert!(catalog.remove_namespace(namespace_id).is_some());
	/// ```
	pub fn remove_namespace(
		&mut self,
		namespacae_id: NamespaceId,
	) -> Option<Namespace<'loc>> {
		self.namespaces.remove(namespacae_id.key())
	}

	/// Returns `true` if the [`Namespace`] of the given [`NamespaceId`] exists.
	///
	/// [`Namespace`]: struct.Namespace.html
	/// [`NamespaceId`]: struct.NamespaceId.html
	///
	/// # Example
	///
	/// ```
	/// # extern crate astral;
	/// use astral::resource::assets::{Catalog, Namespace};
	///
	/// let mut catalog = Catalog::new();
	///
	/// let namespace_id = catalog.add_namespace(Namespace::new());
	/// assert!(catalog.contains_namespace(namespace_id));
	/// ```
	pub fn contains_namespace(&self, namespace_id: NamespaceId) -> bool {
		self.namespaces.contains_key(namespace_id.key())
	}

	/// Returns an [`Iterator`] over all [`NamespaceId`]s and [`Namespace`]s in the
	/// `Catalog`.
	///
	/// [`Iterator`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html
	/// [`Namespace`]: struct.Namespace.html
	/// [`NamespaceId`]: struct.NamespaceId.html
	///
	/// # Example
	///
	/// ```
	/// # extern crate astral;
	/// use astral::resource::assets::{Catalog, Namespace};
	///
	/// let mut catalog = Catalog::new();
	///
	/// catalog.add_namespace(Namespace::new());
	/// for (id, namespace) in catalog.iter_namespaces() {
	/// 	// do something with the namespace
	/// }
	/// ```
	pub fn iter_namespaces(
		&self,
	) -> impl Iterator<Item = (NamespaceId, &Namespace<'loc>)> {
		self.namespaces
			.iter()
			.map(|(key, namespace)| (NamespaceId::new(key), namespace))
	}

	/// Opens a file in write-only mode at the given [`VirtualFileSystem`].
	///
	/// See [`Namespace::create`] for more infos.
	///
	/// [`VirtualFileSystem`]: trait.VirtualFileSystem.html
	/// [`Namespace::create`]: struct.Namespace.html#method.create
	///
	/// # Example
	///
	/// ```no_run
	/// # extern crate astral;
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	/// use astral::core::string::Name;
	/// use astral::resource::assets::{Catalog, FileSystem, Location, Namespace};
	///
	/// let mut namespace = Namespace::new();
	/// let cwd_index = namespace.add_virtual_file_system(FileSystem::new(".", false)?)?;
	///
	/// let mut catalog = Catalog::new();
	/// let namespace_id = catalog.add_namespace(namespace);
	///
	/// let location = Location::new(namespace_id, Name::from("file.txt"));
	/// catalog.create(location, Some(cwd_index));
	/// # Ok(())
	/// # }
	/// ```
	pub fn create(
		&mut self,
		location: Location,
		virtual_file_system_index: Option<VirtualFileSystemIndex>,
	) -> Option<Result<impl Write>> {
		self.get_namespace_mut(location.namespace_id)
			.and_then(|namespace| {
				namespace.create(location.name, virtual_file_system_index)
			})
	}

	/// Opens a file in write-only mode at the given [`VirtualFileSystem`].
	///
	/// See [`Namespace::create_new`] for more infos.
	///
	/// [`Namespace::create_new`]: struct.Namespace.html#method.create_new
	/// [`VirtualFileSystem`]: trait.VirtualFileSystem.html
	///
	/// # Example
	///
	/// ```no_run
	/// # extern crate astral;
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	/// use astral::core::string::Name;
	/// use astral::resource::assets::{Catalog, FileSystem, Location, Namespace};
	///
	/// let mut namespace = Namespace::new();
	/// let cwd_index = namespace.add_virtual_file_system(FileSystem::new(".", false)?)?;
	///
	/// let mut catalog = Catalog::new();
	/// let namespace_id = catalog.add_namespace(namespace);
	///
	/// let location = Location::new(namespace_id, Name::from("file.txt"));
	/// catalog.create_new(location, Some(cwd_index));
	/// # Ok(())
	/// # }
	/// ```
	pub fn create_new(
		&mut self,
		location: Location,
		virtual_file_system_index: Option<VirtualFileSystemIndex>,
	) -> Option<Result<impl Write>> {
		self.get_namespace_mut(location.namespace_id)
			.and_then(|namespace| {
				namespace.create_new(location.name, virtual_file_system_index)
			})
	}

	/// Returns whether the `Catalog` is aware of the file and the
	/// entity exists.
	///
	/// See [`Namespace::exists`] for more infos.
	///
	/// [`Namespace::exists`]: struct.Namespace.html#method.exists
	///
	/// # Example
	///
	/// ```no_run
	/// # extern crate astral;
	/// use astral::core::string::Name;
	/// use astral::resource::assets::{Catalog, FileSystem, Location, Namespace};
	///
	/// let mut namespace = Namespace::new();
	///
	/// // fill namespace
	///
	/// let mut catalog = Catalog::new();
	/// let namespace_id = catalog.add_namespace(namespace);
	///
	/// let location = Location::new(namespace_id, Name::from("does_not_exists.txt"));
	///
	/// assert_eq!(catalog.exists(location), false);
	/// ```
	pub fn exists(&self, location: Location) -> bool {
		self.get_namespace(location.namespace_id)
			.map_or(false, |namespace| namespace.exists(location.name))
	}

	/// Returns the last modification time of the file if the `Catalog`
	/// is aware of it.
	///
	/// See [`Namespace::modified`] for more infos.
	///
	/// [`Namespace::modified`]: struct.Namespace.html#method.modified
	///
	/// # Example
	///
	/// ```no_run
	/// # extern crate astral;
	/// use astral::core::string::Name;
	/// use astral::resource::assets::{Catalog, FileSystem, Location, Namespace};
	///
	/// let mut namespace = Namespace::new();
	///
	/// // fill namespace
	///
	/// let mut catalog = Catalog::new();
	/// let namespace_id = catalog.add_namespace(namespace);
	///
	/// let location = Location::new(namespace_id, Name::from("file.txt"));
	///
	/// println!("{:?}", catalog.modified(location));
	/// ```
	pub fn modified(&self, location: Location) -> Option<Result<SystemTime>> {
		self.get_namespace(location.namespace_id)
			.and_then(|namespace| namespace.modified(location.name))
	}

	/// Opens the file in read-only mode. Returns [`None`], if the `Catalog` is
	/// not aware of it.
	///
	/// See [`Namespace::open`] for more infos.
	///
	/// [`Namespace::open`]: struct.Namespace.html#method.open
	/// [`None`]: https://doc.rust-lang.org/std/option/enum.Option.html#variant.None
	///
	/// # Example
	///
	/// ```no_run
	/// # extern crate astral;
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	/// use astral::core::string::Name;
	/// use astral::resource::assets::{Catalog, FileSystem, Location, Namespace};
	///
	/// let mut namespace = Namespace::new();
	/// let cwd_index = namespace.add_virtual_file_system(FileSystem::new(".", false)?)?;
	///
	/// let mut catalog = Catalog::new();
	/// let namespace_id = catalog.add_namespace(namespace);
	///
	/// let location = Location::new(namespace_id, Name::from("file.txt"));
	/// if let Some(read) = catalog.open(location) {
	/// 	let file = read?;
	/// }
	/// # Ok(())
	/// # }
	/// ```
	pub fn open(&self, location: Location) -> Option<Result<impl Read>> {
		self.get_namespace(location.namespace_id)
			.and_then(|namespace| namespace.open(location.name))
	}

	/// Remove the file. Returns [`Some`]`(`[`Result`]`<()>)`, if the `Catalog`
	/// is aware of the file. [`Result`] determines if the removal was successful.
	/// Returns [`None`] otherwise
	///
	/// See [`Namespace::remove`] for more infos.
	///
	/// [`Namespace::remove`]: struct.Namespace.html#method.remove
	/// [`None`]: https://doc.rust-lang.org/std/option/enum.Option.html#variant.None
	///
	/// # Example
	///
	/// ```no_run
	/// # extern crate astral;
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	/// use astral::core::string::Name;
	/// use astral::resource::assets::{Catalog, FileSystem, Location, Namespace};
	///
	/// let mut namespace = Namespace::new();
	/// let cwd_index = namespace.add_virtual_file_system(FileSystem::new(".", false)?)?;
	///
	/// let mut catalog = Catalog::new();
	/// let namespace_id = catalog.add_namespace(namespace);
	///
	/// let location = Location::new(namespace_id, Name::from("file.txt"));
	/// if let Some(result) = catalog.remove(location) {
	/// 	println!("removing file: {:?}", result);
	/// }
	/// # Ok(())
	/// # }
	/// ```
	pub fn remove(&mut self, location: Location) -> Option<Result<()>> {
		self.get_namespace_mut(location.namespace_id)
			.and_then(|namespace| namespace.remove(location.name))
	}
}

impl<'loc> Debug for Catalog<'loc> {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		fmt.debug_map().entries(self.iter_namespaces()).finish()
	}
}

impl<'loc> Index<NamespaceId> for Catalog<'loc> {
	type Output = Namespace<'loc>;

	fn index(&self, namespace_id: NamespaceId) -> &Self::Output {
		self.get_namespace(namespace_id)
			.expect("Invalid namespace id")
	}
}

impl<'loc> IndexMut<NamespaceId> for Catalog<'loc> {
	fn index_mut(&mut self, namespace_id: NamespaceId) -> &mut Self::Output {
		self.get_namespace_mut(namespace_id)
			.expect("Invalid namespace id")
	}
}
