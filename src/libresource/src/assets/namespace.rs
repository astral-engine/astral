// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::{
	boxed::Box,
	collections::hash_map::{self, HashMap},
	fmt::{self, Debug, Formatter},
	hash::BuildHasherDefault,
	io::{Read, Write},
	time::SystemTime,
};

use astral_core::{collections::SparseSlotMap, error::OptionExt, hash::NopHasher, string::Name};

use super::{Error, ErrorKind, Result, VirtualFileSystem, VirtualFileSystemIndex};

/// A `Namespace` contains multiple, various [`VirtualFileSystem`]s.
///
/// The files inside of a [`VirtualFileSystem`] are cached for a faster
/// access. The cache can be recreated with [`reload`].
///
/// [`VirtualFileSystem`]: trait.VirtualFileSystem.html
/// [`reload`]: #method.reload
#[derive(Default)]
pub struct Namespace<'loc> {
	virtual_file_systems: SparseSlotMap<Box<dyn VirtualFileSystem + 'loc>>,
	paths: HashMap<Name, VirtualFileSystemIndex, BuildHasherDefault<NopHasher>>,
}

impl<'loc> Namespace<'loc> {
	/// Construct a new empty `Namespace`.
	///
	/// # Example
	///
	/// ```
	/// use astral::resource::assets::Namespace;
	///
	/// let namespace = Namespace::new();
	/// assert!(namespace.is_empty());
	/// ```
	pub fn new() -> Self {
		Self::default()
	}

	/// Construct a new, empty `Namespace` with the specified capacity.
	///
	/// The `Namespace` will be able to hold exactly `virtual_file_systems` elements
	/// file systems and `files` unique files without reallocating.
	/// If both parameters are 0, the `Namespace` will not allocate.
	///
	/// # Example
	///
	/// ```
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::with_capacity(1, 100);
	///
	/// // The namespace contains no items, even though it has capacity for more
	/// assert_eq!(namespace.virtual_file_systems(), 0);
	///
	/// // This can be done without reallocating,
	/// // assuming there are no more than 100 files...
	/// namespace.add_virtual_file_system(FileSystem::new(".", false)?);
	///
	/// // ...but this may make the namespace reallocate
	/// namespace.add_virtual_file_system(FileSystem::new("..", false)?);
	/// # Ok(())
	/// # }
	/// ```
	pub fn with_capacity(virtual_file_systems: usize, files: usize) -> Self {
		Self {
			virtual_file_systems: SparseSlotMap::with_capacity(virtual_file_systems),
			paths: HashMap::with_capacity_and_hasher(files, BuildHasherDefault::default()),
		}
	}

	/// Returns the number of [`VirtualFileSystem`]s in the `Namespace`.
	///
	/// [`VirtualFileSystem`]: trait.VirtualFileSystem.html
	///
	/// # Example
	///
	/// ```
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::with_capacity(1, 100);
	///
	/// assert_eq!(namespace.virtual_file_systems(), 0);
	/// namespace.add_virtual_file_system(FileSystem::new(".", false)?);
	/// assert_eq!(namespace.virtual_file_systems(), 1);
	/// # Ok(())
	/// # }
	/// ```
	pub fn virtual_file_systems(&self) -> usize {
		self.virtual_file_systems.len() as usize
	}

	/// Returns the number of files in the `Namespace`.
	///
	/// # Example
	///
	/// ```
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	///
	/// assert_eq!(namespace.files(), 0);
	/// namespace.add_virtual_file_system(FileSystem::new(".", false)?);
	/// assert!(namespace.files() > 0);
	/// # Ok(())
	/// # }
	/// ```
	pub fn files(&self) -> usize {
		self.paths.len()
	}

	/// Returns `true` if the `Namespace` contains no [`VirtualFileSystem`]s or no files.
	///
	/// [`VirtualFileSystem`]: trait.VirtualFileSystem.html
	///
	/// # Example
	///
	/// ```
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	///
	/// assert!(namespace.is_empty());
	/// namespace.add_virtual_file_system(FileSystem::new(".", false)?);
	/// assert!(!namespace.is_empty());
	/// # Ok(())
	/// # }
	/// ```
	pub fn is_empty(&self) -> bool {
		self.virtual_file_systems() == 0 || self.files() == 0
	}

	/// Adds a new [`VirtualFileSystem`] and returns its [`VirtualFileSystemIndex`]
	/// to query it at a later time.
	///
	/// [`VirtualFileSystem`]: trait.VirtualFileSystem.html
	/// [`VirtualFileSystemIndex`]: struct.VirtualFileSystemIndex.html
	///
	/// # Example
	///
	/// ```no_run
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	/// let cwd_index = namespace.add_virtual_file_system(FileSystem::new(".", false)?)?;
	/// # Ok(())
	/// # }
	/// ```
	pub fn add_virtual_file_system(
		&mut self,
		virtual_file_system: impl Into<Box<dyn VirtualFileSystem + 'loc>>,
	) -> Result<VirtualFileSystemIndex> {
		let virtual_file_system = virtual_file_system.into();
		let index = VirtualFileSystemIndex::new(self.virtual_file_systems.create_key());

		for path in virtual_file_system.iter()? {
			let _ = self.paths.insert(path, index);
		}
		let _ = self
			.virtual_file_systems
			.insert_with_key(index.key(), virtual_file_system)
			.map_err(|_| {
				Error::new(
					ErrorKind::Other,
					"Virtual file system could not be inserted",
				)
			})?;

		Ok(index)
	}

	/// Removes a [`VirtualFileSystem`] by its index, which where returned by
	/// [`add_virtual_file_system`]. Returns the file system if any.
	///
	/// [`VirtualFileSystem`]: trait.VirtualFileSystem.html
	/// [`add_virtual_file_system`]: #method.add_virtual_file_system
	///
	/// # Example
	///
	/// ```
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	/// assert!(namespace.is_empty());
	/// let cwd_index = namespace.add_virtual_file_system(FileSystem::new(".", false)?)?;
	/// assert!(!namespace.is_empty());
	/// let vfs = namespace.remove_virtual_file_system(cwd_index);
	/// assert!(namespace.is_empty());
	/// assert!(vfs.is_some());
	/// # Ok(())
	/// # }
	/// ```
	pub fn remove_virtual_file_system(
		&mut self,
		virtual_file_system_index: VirtualFileSystemIndex,
	) -> Option<Box<dyn VirtualFileSystem + 'loc>> {
		self.paths
			.retain(|_, index| *index != virtual_file_system_index);
		self.virtual_file_systems
			.remove(virtual_file_system_index.key())
	}

	/// Removes all [`VirtualFileSystem`]s from the `Namespace`.
	///
	/// [`VirtualFileSystem`]: trait.VirtualFileSystem.html
	///
	/// # Example
	///
	/// ```
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	///
	/// namespace.add_virtual_file_system(FileSystem::new(".", false)?);
	/// namespace.add_virtual_file_system(FileSystem::new("..", false)?);
	/// assert_eq!(namespace.virtual_file_systems(), 2);
	/// namespace.clear();
	/// assert!(namespace.is_empty());
	/// # Ok(())
	/// # }
	/// ```
	pub fn clear(&mut self) {
		self.paths.clear();
		self.virtual_file_systems.clear();
	}

	/// Reloads the [`VirtualFileSystem`] at the given index and updates the
	/// internal cache. This may take some time.
	///
	/// [`VirtualFileSystem`]: trait.VirtualFileSystem.html
	///
	/// # Example
	///
	/// ```no_run
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	///
	/// let cwd_index = namespace.add_virtual_file_system(FileSystem::new(".", false)?)?;
	/// namespace.reload(cwd_index);
	/// # Ok(())
	/// # }
	/// ```
	pub fn reload(&mut self, virtual_file_system_index: VirtualFileSystemIndex) -> Result<()> {
		let virtual_file_system = self
			.virtual_file_systems
			.get(virtual_file_system_index.key())
			.ok_or_error(
				ErrorKind::InvalidIndex,
				format!(
					"Could not get virtual file system for index {:?}",
					virtual_file_system_index
				),
			)?;

		self.paths
			.retain(|_, index| *index != virtual_file_system_index);
		for path in virtual_file_system.iter()? {
			let _ = self.paths.insert(path, virtual_file_system_index);
		}

		Ok(())
	}

	fn get_virtual_file_system(&self, name: Name) -> Option<&dyn VirtualFileSystem> {
		let virtual_file_system_id = self.paths.get(&name)?;
		self.virtual_file_systems
			.get(virtual_file_system_id.key())
			.map(Box::as_ref)
	}

	fn get_virtual_file_system_mut(
		&mut self,
		name: Name,
	) -> Option<&mut (dyn VirtualFileSystem + 'loc)> {
		let virtual_file_system_id = self.paths.get(&name)?;
		self.virtual_file_systems
			.get_mut(virtual_file_system_id.key())
			.map(Box::as_mut)
	}

	/// Returns an [`Iterator`] over all [`VirtualFileSystem`]s in the `Namespace`.
	///
	/// [`Iterator`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html
	/// [`VirtualFileSystem`]: trait.VirtualFileSystem.html
	///
	/// # Example
	///
	/// ```
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	///
	/// namespace.add_virtual_file_system(FileSystem::new(".", false)?);
	/// for vfs in namespace.iter() {
	/// 	// do something with the file systems
	/// }
	/// # Ok(())
	/// # }
	/// ```
	pub fn iter(&self) -> impl Iterator<Item = (&dyn VirtualFileSystem, Name)> {
		Iter {
			virtual_file_systems: &self.virtual_file_systems,
			paths: self.paths.iter(),
		}
	}

	/// Opens a file in write-only mode at the given [`VirtualFileSystem`].
	///
	/// If no [`VirtualFileSystemIndex`] is provided, the first [`VirtualFileSystem`]
	/// will be used, which is not read-only.
	///
	/// This function will create a file if it does not exist, and will truncate
	/// it if it does.
	///
	/// [`VirtualFileSystem`]: trait.VirtualFileSystem.html
	/// [`VirtualFileSystemIndex`]: trait.VirtualFileSystemIndex.html
	///
	/// # Example
	///
	/// ```no_run
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	/// use astral::core::string::Name;
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	///
	/// let cwd_index = namespace.add_virtual_file_system(FileSystem::new(".", false)?)?;
	/// namespace.create(Name::from("a.txt"), Some(cwd_index));
	/// # Ok(())
	/// # }
	/// ```
	pub fn create(
		&mut self,
		name: Name,
		virtual_file_system_index: Option<VirtualFileSystemIndex>,
	) -> Option<Result<impl Write>> {
		let (index, vfs) = if let Some(index) = virtual_file_system_index {
			(
				index,
				self.virtual_file_systems
					.get_mut(index.key())
					.filter(|vfs| !vfs.readonly())?,
			)
		} else {
			self.virtual_file_systems
				.iter_mut()
				.map(|(index, entry)| (VirtualFileSystemIndex::new(index), entry))
				.find(|(_, vfs)| !vfs.readonly())?
		};
		let write = Some(vfs.create(name));
		let _ = self.paths.insert(name, index);
		write
	}

	/// Creates a file in write-only mode at the given [`VirtualFileSystem`].
	///
	/// If no [`VirtualFileSystemIndex`] is provided, the first [`VirtualFileSystem`]
	/// will be used, which is not read-only.
	///
	/// No file is allowed to exist at the target location, also no (dangling) symlink.
	///
	/// [`VirtualFileSystem`]: trait.VirtualFileSystem.html
	/// [`VirtualFileSystemIndex`]: trait.VirtualFileSystemIndex.html
	///
	/// # Example
	///
	/// ```no_run
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	/// use astral::core::string::Name;
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	///
	/// let cwd_index = namespace.add_virtual_file_system(FileSystem::new(".", false)?)?;
	/// namespace.create_new(Name::from("a.txt"), Some(cwd_index));
	/// # Ok(())
	/// # }
	/// ```
	pub fn create_new(
		&mut self,
		name: Name,
		virtual_file_system_index: Option<VirtualFileSystemIndex>,
	) -> Option<Result<impl Write>> {
		let (index, vfs) = if let Some(index) = virtual_file_system_index {
			(
				index,
				self.virtual_file_systems
					.get_mut(index.key())
					.filter(|vfs| !vfs.readonly())?,
			)
		} else {
			self.virtual_file_systems
				.iter_mut()
				.map(|(index, entry)| (VirtualFileSystemIndex::new(index), entry))
				.find(|(_, vfs)| !vfs.readonly())?
		};
		let write = Some(vfs.create_new(name));
		let _ = self.paths.insert(name, index);
		write
	}

	/// Returns whether the `Namespace` is aware of the file and the
	/// entity exists.
	///
	/// # Example
	///
	/// ```no_run
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	/// use astral::core::string::Name;
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	///
	/// let cwd_index = namespace.add_virtual_file_system(FileSystem::new(".", false)?)?;
	/// assert_eq!(namespace.exists(Name::from("does_not_exist.txt")), false);
	/// # Ok(())
	/// # }
	/// ```
	pub fn exists(&self, name: Name) -> bool {
		self.get_virtual_file_system(name)
			.map_or(false, |virtual_file_system| {
				virtual_file_system.exists(name)
			})
	}

	/// Returns the last modification time of the file if the `Namespace`
	/// is aware of it.
	///
	/// # Example
	///
	/// ```no_run
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	/// use astral::core::string::Name;
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	///
	/// let cwd_index = namespace.add_virtual_file_system(FileSystem::new(".", false)?)?;
	/// println!("{:?}", namespace.modified(Name::from("file.txt")));
	/// # Ok(())
	/// # }
	/// ```
	pub fn modified(&self, name: Name) -> Option<Result<SystemTime>> {
		self.get_virtual_file_system(name)
			.map(|virtual_file_system| Ok(virtual_file_system.modified(name)?))
	}

	/// Opens the file in read-only mode. Returns [`None`], if the `Namespace` is
	/// not aware of it.
	///
	/// [`None`]: https://doc.rust-lang.org/std/option/enum.Option.html#variant.None
	///
	/// # Example
	///
	/// ```no_run
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	/// use astral::core::string::Name;
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	///
	/// let cwd_index = namespace.add_virtual_file_system(FileSystem::new(".", false)?)?;
	/// if let Some(read) = namespace.open(Name::from("file.txt")) {
	/// 	let file = read?;
	/// }
	/// # Ok(())
	/// # }
	/// ```
	pub fn open(&self, name: Name) -> Option<Result<impl Read>> {
		self.get_virtual_file_system(name)
			.map(|virtual_file_system| Ok(virtual_file_system.open(name)?))
	}

	/// Remove the file. Returns [`Some`]`(`[`Result`]`<()>)`, if the `Namespace`
	/// is aware of the file. [`Result`] determines if the removal was successful.
	/// Returns [`None`] otherwise
	///
	/// [`None`]: https://doc.rust-lang.org/std/option/enum.Option.html#variant.None
	/// [`Some`]: https://doc.rust-lang.org/std/option/enum.Option.html#variant.Some
	/// [`Result`]: https://doc.rust-lang.org/std/option/enum.Result.html
	///
	/// # Example
	///
	/// ```no_run
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	/// use astral::core::string::Name;
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	///
	/// let cwd_index = namespace.add_virtual_file_system(FileSystem::new(".", false)?)?;
	/// if let Some(result) = namespace.remove(Name::from("file.txt")) {
	/// 	println!("removing file: {:?}", result);
	/// }
	/// # Ok(())
	/// # }
	/// ```
	pub fn remove(&mut self, name: Name) -> Option<Result<()>> {
		{
			let virtual_file_system = self.get_virtual_file_system_mut(name)?;

			if virtual_file_system.readonly() {
				return None;
			} else if let Err(err) = virtual_file_system.remove(name) {
				return Some(Err(err));
			}
		}
		let _ = self.paths.remove(&name);
		Some(Ok(()))
	}
}

#[derive(Debug)]
struct Iter<'loc> {
	virtual_file_systems: &'loc SparseSlotMap<Box<dyn VirtualFileSystem + 'loc>>,
	paths: hash_map::Iter<'loc, Name, VirtualFileSystemIndex>,
}

impl<'loc> Iterator for Iter<'loc> {
	type Item = (&'loc dyn VirtualFileSystem, Name);

	fn next(&mut self) -> Option<Self::Item> {
		let (name, index) = self.paths.next()?;
		let virtual_file_system = self.virtual_file_systems.get(index.key())?;
		Some((virtual_file_system.as_ref(), *name))
	}
}

impl Debug for Namespace<'_> {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		fmt.debug_map()
			.entries(self.iter().map(|(location, path)| (location.name(), path)))
			.finish()
	}
}
