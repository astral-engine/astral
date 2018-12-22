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

use astral_core::{
	collections::SparseSlotMap,
	error::OptionExt,
	hash::{Murmur3, NopHasher},
	string::Name,
};

use super::{Error, ErrorKind, Result, VirtualFileSystem, VirtualFileSystemIndex};

/// A `Namespace` contains multiple, various [`VirtualFileSystem`]s.
///
/// The files inside of a [`VirtualFileSystem`] are cached for a faster
/// access. The cache can be recreated with [`reload`].
///
/// [`VirtualFileSystem`]: trait.VirtualFileSystem.html
/// [`reload`]: #method.reload
pub struct Namespace<'str, 'vfs, H = BuildHasherDefault<Murmur3>> {
	virtual_file_systems: SparseSlotMap<Box<dyn VirtualFileSystem<'str, H> + 'vfs>>,
	paths: HashMap<Name<'str, H>, VirtualFileSystemIndex, BuildHasherDefault<NopHasher>>,
}

impl<'str, 'vfs, H> Namespace<'str, 'vfs, H> {
	/// Construct a new empty `Namespace`.
	///
	/// # Example
	///
	/// ```
	/// use astral::resource::assets::Namespace;
	///
	/// let namespace: Namespace<'_, '_> = Namespace::new();
	/// assert!(namespace.is_empty());
	/// ```
	pub fn new() -> Self {
		Self {
			virtual_file_systems: SparseSlotMap::default(),
			paths: HashMap::default(),
		}
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
	/// # use astral::{third_party::slog, Engine, core::{self, string}, resource::{self, assets}};
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let core_system = core::System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &core_system);
	///	# let resource_system = resource::System::new(&engine);
	///	# let asset_subsystem = assets::Subsystem::new(&resource_system);
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace: Namespace<'_, '_> = Namespace::with_capacity(1, 100);
	///
	/// // The namespace contains no items, even though it has capacity for more
	/// assert_eq!(namespace.virtual_file_systems(), 0);
	///
	/// // This can be done without reallocating,
	/// // assuming there are no more than 100 files...
	/// let file_system = FileSystem::new(".", &asset_subsystem, &string_subsystem)?;
	/// namespace.add_virtual_file_system(file_system)?;
	///
	/// // ...but this may make the namespace reallocate
	/// let file_system = FileSystem::new("..", &asset_subsystem, &string_subsystem)?;
	/// namespace.add_virtual_file_system(file_system)?;
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
	/// # use astral::{third_party::slog, Engine, core::{self, string}, resource::{self, assets}};
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let core_system = core::System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &core_system);
	///	# let resource_system = resource::System::new(&engine);
	///	# let asset_subsystem = assets::Subsystem::new(&resource_system);
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::default();
	///
	/// assert_eq!(namespace.virtual_file_systems(), 0);
	/// let file_system = FileSystem::new(".", &asset_subsystem, &string_subsystem)?;
	/// namespace.add_virtual_file_system(file_system)?;
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
	/// # use astral::{third_party::slog, Engine, core::{self, string}, resource::{self, assets}};
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let core_system = core::System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &core_system);
	///	# let resource_system = resource::System::new(&engine);
	///	# let asset_subsystem = assets::Subsystem::new(&resource_system);
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::default();
	///
	/// assert_eq!(namespace.files(), 0);
	/// let file_system = FileSystem::new(".", &asset_subsystem, &string_subsystem)?;
	/// namespace.add_virtual_file_system(file_system)?;
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
	/// # use astral::{third_party::slog, Engine, core::{self, string}, resource::{self, assets}};
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let core_system = core::System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &core_system);
	///	# let resource_system = resource::System::new(&engine);
	///	# let asset_subsystem = assets::Subsystem::new(&resource_system);
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::default();
	///
	/// assert!(namespace.is_empty());
	/// let file_system = FileSystem::new(".", &asset_subsystem, &string_subsystem)?;
	/// namespace.add_virtual_file_system(file_system)?;
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
	/// # use astral::{third_party::slog, Engine, core::{self, string}, resource::{self, assets}};
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let core_system = core::System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &core_system);
	///	# let resource_system = resource::System::new(&engine);
	///	# let asset_subsystem = assets::Subsystem::new(&resource_system);
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	/// let file_system = FileSystem::new(".", &asset_subsystem, &string_subsystem)?;
	/// namespace.add_virtual_file_system(file_system)?;
	/// # Ok(())
	/// # }
	/// ```
	pub fn add_virtual_file_system(
		&mut self,
		virtual_file_system: impl Into<Box<dyn VirtualFileSystem<'str, H> + 'vfs>>,
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
	/// # use astral::{third_party::slog, Engine, core::{self, string}, resource::{self, assets}};
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let core_system = core::System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &core_system);
	///	# let resource_system = resource::System::new(&engine);
	///	# let asset_subsystem = assets::Subsystem::new(&resource_system);
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	/// assert!(namespace.is_empty());
	/// let file_system = FileSystem::new(".", &asset_subsystem, &string_subsystem)?;
	/// let cwd_index = namespace.add_virtual_file_system(file_system)?;
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
	) -> Option<Box<dyn VirtualFileSystem<'str, H> + 'vfs>> {
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
	/// # use astral::{third_party::slog, Engine, core::{self, string}, resource::{self, assets}};
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let core_system = core::System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &core_system);
	///	# let resource_system = resource::System::new(&engine);
	///	# let asset_subsystem = assets::Subsystem::new(&resource_system);
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	///
	/// let file_system1 = FileSystem::new(".", &asset_subsystem, &string_subsystem)?;
	/// let file_system2 = FileSystem::new("..", &asset_subsystem, &string_subsystem)?;
	/// namespace.add_virtual_file_system(file_system1)?;
	/// namespace.add_virtual_file_system(file_system2)?;
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
	/// # use astral::{third_party::slog, Engine, core::{self, string}, resource::{self, assets}};
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let core_system = core::System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &core_system);
	///	# let resource_system = resource::System::new(&engine);
	///	# let asset_subsystem = assets::Subsystem::new(&resource_system);
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	///
	/// let file_system = FileSystem::new(".", &asset_subsystem, &string_subsystem)?;
	/// let cwd_index = namespace.add_virtual_file_system(file_system)?;
	/// namespace.reload(cwd_index)?;
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

	fn get_virtual_file_system(
		&self,
		name: Name<'str, H>,
	) -> Option<&dyn VirtualFileSystem<'str, H>> {
		let virtual_file_system_id = self.paths.get(&name)?;
		self.virtual_file_systems
			.get(virtual_file_system_id.key())
			.map(Box::as_ref)
	}

	fn get_virtual_file_system_mut(
		&mut self,
		name: Name<'str, H>,
	) -> Option<&mut (dyn VirtualFileSystem<'str, H> + 'vfs)> {
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
	/// # use astral::{third_party::slog, Engine, core::{self, string}, resource::{self, assets}};
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let core_system = core::System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &core_system);
	///	# let resource_system = resource::System::new(&engine);
	///	# let asset_subsystem = assets::Subsystem::new(&resource_system);
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	///
	/// let file_system = FileSystem::new(".", &asset_subsystem, &string_subsystem)?;
	/// namespace.add_virtual_file_system(file_system)?;
	/// # #[allow(unused_variables)]
	/// for vfs in namespace.iter() {
	/// 	// do something with the file systems
	/// }
	/// # Ok(())
	/// # }
	/// ```
	pub fn iter(&self) -> impl Iterator<Item = (&dyn VirtualFileSystem<'str, H>, Name<'str, H>)> {
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
	/// # use astral::{third_party::slog, Engine, core::{self, string}, resource::{self, assets}};
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let core_system = core::System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &core_system);
	///	# let resource_system = resource::System::new(&engine);
	///	# let asset_subsystem = assets::Subsystem::new(&resource_system);
	/// use astral::core::string::Name;
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	///
	/// let file_system = FileSystem::new(".", &asset_subsystem, &string_subsystem)?;
	/// let cwd_index = namespace.add_virtual_file_system(file_system)?;
	/// let file_name = Name::new("a.txt", &string_subsystem);
	/// namespace.create(file_name, Some(cwd_index));
	/// # Ok(())
	/// # }
	/// ```
	pub fn create(
		&mut self,
		name: Name<'str, H>,
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
	/// # use astral::{third_party::slog, Engine, core::{self, string}, resource::{self, assets}};
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let core_system = core::System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &core_system);
	///	# let resource_system = resource::System::new(&engine);
	///	# let asset_subsystem = assets::Subsystem::new(&resource_system);
	/// use astral::core::string::Name;
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	///
	/// let file_system = FileSystem::new(".", &asset_subsystem, &string_subsystem)?;
	/// let cwd_index = namespace.add_virtual_file_system(file_system)?;
	/// let file_name = Name::new("a.txt", &string_subsystem);
	/// namespace.create_new(file_name, Some(cwd_index));
	/// # Ok(())
	/// # }
	/// ```
	pub fn create_new(
		&mut self,
		name: Name<'str, H>,
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
	/// # use astral::{third_party::slog, Engine, core::{self, string}, resource::{self, assets}};
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let core_system = core::System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &core_system);
	///	# let resource_system = resource::System::new(&engine);
	///	# let asset_subsystem = assets::Subsystem::new(&resource_system);
	/// use astral::core::string::Name;
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	///
	/// let file_system = FileSystem::new(".", &asset_subsystem, &string_subsystem)?;
	/// namespace.add_virtual_file_system(file_system)?;
	/// let file_name = Name::new("does_not_exist.txt", &string_subsystem);
	/// assert_eq!(namespace.exists(file_name), false);
	/// # Ok(())
	/// # }
	/// ```
	pub fn exists(&self, name: Name<'str, H>) -> bool {
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
	/// # use astral::{third_party::slog, Engine, core::{self, string}, resource::{self, assets}};
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let core_system = core::System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &core_system);
	///	# let resource_system = resource::System::new(&engine);
	///	# let asset_subsystem = assets::Subsystem::new(&resource_system);
	/// use astral::core::string::Name;
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	///
	/// let file_system = FileSystem::new(".", &asset_subsystem, &string_subsystem)?;
	/// namespace.add_virtual_file_system(file_system)?;
	/// let file_name = Name::new("file.txt", &string_subsystem);
	/// println!("{:?}", namespace.modified(Name::from(file_name)));
	/// # Ok(())
	/// # }
	/// ```
	pub fn modified(&self, name: Name<'str, H>) -> Option<Result<SystemTime>> {
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
	/// # use astral::{third_party::slog, Engine, core::{self, string}, resource::{self, assets}};
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let core_system = core::System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &core_system);
	///	# let resource_system = resource::System::new(&engine);
	///	# let asset_subsystem = assets::Subsystem::new(&resource_system);
	/// use astral::core::string::Name;
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	///
	/// let file_system = FileSystem::new(".", &asset_subsystem, &string_subsystem)?;
	/// namespace.add_virtual_file_system(file_system)?;
	/// let file_name = Name::new("file.txt", &string_subsystem);
	/// if let Some(read) = namespace.open(file_name) {
	///     # #[allow(unused_variables)]
	/// 	let file = read?;
	/// }
	/// # Ok(())
	/// # }
	/// ```
	pub fn open(&self, name: Name<'str, H>) -> Option<Result<impl Read>> {
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
	/// # use astral::{third_party::slog, Engine, core::{self, string}, resource::{self, assets}};
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	///	# let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	# let engine = Engine::new(&logger);
	///	# let core_system = core::System::new(&engine);
	///	# let string_subsystem = string::Subsystem::new(64, &core_system);
	///	# let resource_system = resource::System::new(&engine);
	///	# let asset_subsystem = assets::Subsystem::new(&resource_system);
	/// use astral::core::string::Name;
	/// use astral::resource::assets::{Namespace, FileSystem};
	///
	/// let mut namespace = Namespace::new();
	///
	/// let file_system = FileSystem::new(".", &asset_subsystem, &string_subsystem)?;
	/// namespace.add_virtual_file_system(file_system)?;
	/// let file_name = Name::new("file.txt", &string_subsystem);
	/// if let Some(result) = namespace.remove(file_name) {
	/// 	println!("removing file: {:?}", result);
	/// }
	/// # Ok(())
	/// # }
	/// ```
	pub fn remove(&mut self, name: Name<'str, H>) -> Option<Result<()>> {
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

impl Default for Namespace<'_, '_, BuildHasherDefault<Murmur3>> {
	fn default() -> Self {
		Self {
			virtual_file_systems: SparseSlotMap::default(),
			paths: HashMap::default(),
		}
	}
}

#[derive(Debug)]
struct Iter<'str, 'vfs, H> {
	virtual_file_systems: &'vfs SparseSlotMap<Box<dyn VirtualFileSystem<'str, H> + 'vfs>>,
	paths: hash_map::Iter<'vfs, Name<'str, H>, VirtualFileSystemIndex>,
}

impl<'str, 'vfs, H> Iterator for Iter<'str, 'vfs, H> {
	type Item = (&'vfs dyn VirtualFileSystem<'str, H>, Name<'str, H>);

	fn next(&mut self) -> Option<Self::Item> {
		let (name, index) = self.paths.next()?;
		let virtual_file_system = self.virtual_file_systems.get(index.key())?;
		Some((virtual_file_system.as_ref(), *name))
	}
}

impl<H> Debug for Namespace<'_, '_, H> {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		fmt.debug_map()
			.entries(self.iter().map(|(location, path)| (location.name(), path)))
			.finish()
	}
}
