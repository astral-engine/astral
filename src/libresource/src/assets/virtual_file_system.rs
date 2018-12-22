// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use super::Result;

use std::{
	fmt::Debug,
	hash::BuildHasherDefault,
	io::{Read, Write},
	time::SystemTime,
};

use astral_core::{hash::Murmur3, string::Name};

/// A virtual file system is an abstraction to a concrete file system with which
/// you can read, write and create files.
///
/// The most primitive file system is the file system of the operating system.
/// Therefore an implementation is given with [`FileSystem`].
///
/// [`FileSystem`]: struct.FileSystem.html
#[cfg_attr(unstable, doc(spotlight))]
pub trait VirtualFileSystem<'str, H = BuildHasherDefault<Murmur3>>: Debug + Send + Sync {
	/// Returns the [`Name`] of the file system.
	///
	/// [`Name`]: ../../core/string/struct.Name.html
	fn name(&self) -> Name<'str, H>;
	/// Returns if the file system is read-only.
	fn readonly(&self) -> bool;
	/// Returns an [`Iterator`] over all files in the file system.
	///
	/// [`Iterator`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html
	fn iter<'a>(&'a self) -> Result<Box<dyn Iterator<Item = Name<'str, H>> + 'a>>;

	/// Opens a file in write-only mode.
	///
	/// This function will create a file if it does not exist, and will truncate
	/// it if it does.
	fn create(&mut self, path: Name<'str, H>) -> Result<Box<dyn Write>>;

	/// Creates a file in write-only mode.
	///
	/// No file is allowed to exist at the target location, also no (dangling) symlink.
	fn create_new(&mut self, path: Name<'str, H>) -> Result<Box<dyn Write>>;

	/// Returns whether the path points at an existing entity.
	fn exists(&self, path: Name<'str, H>) -> bool;

	/// Returns the last modification time at this entity.
	fn modified(&self, path: Name<'str, H>) -> Result<SystemTime>;

	/// Attempts to open a file in read-only mode.
	fn open(&self, path: Name<'str, H>) -> Result<Box<dyn Read>>;

	/// Removes a file from the filesystem.
	fn remove(&mut self, path: Name<'str, H>) -> Result<()>;
}

#[allow(clippy::use_self)]
impl<'str, 'loc, L> From<L> for Box<dyn VirtualFileSystem<'str> + 'loc>
where
	L: VirtualFileSystem<'str> + 'loc,
{
	fn from(location: L) -> Self {
		Box::new(location)
	}
}
