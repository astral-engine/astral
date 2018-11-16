// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use std::{
	boxed::Box,
	fs::{self, OpenOptions},
	io::{Read, Write},
	path::{Component, PathBuf},
	time::SystemTime,
};

use astral_core::{
	error::ResultExt,
	log::{error, warn},
	string::Name,
};

use super::{ErrorKind, Result, VirtualFileSystem};

/// A `FileSystem` is a view into the systems file system.
#[derive(Debug, Serialize, Deserialize)]
pub struct FileSystem {
	root: PathBuf,
	recursive: bool,
}

impl FileSystem {
	/// Construct a new `FileSystem` at the specified root path.
	/// If wished, the file system can search files in a recursive manner.
	///
	/// # Example
	///
	/// ```no_run
	/// # extern crate astral;
	/// # fn main() -> Result<(), astral::resource::assets::Error> {
	/// use astral::resource::assets::{FileSystem, VirtualFileSystem};
	///
	/// let file_system = FileSystem::new(".", false)?;
	/// let file = file_system.open("a.txt".into())?;
	/// # Ok(())
	/// # }
	/// ```
	pub fn new<P: Into<PathBuf>>(root: P, recursive: bool) -> Result<Self> {
		Ok(Self {
			root: root.into(),
			recursive,
		})
	}

	/// test func
	pub fn test() -> Self {
		unimplemented!()
	}

	fn concat_path(&self, path: Name) -> PathBuf {
		let mut path_buf = self.root.clone();
		path_buf.push(path.to_string());
		path_buf
	}
}

impl VirtualFileSystem for FileSystem {
	fn name(&self) -> Name {
		self.root.to_string_lossy().into()
	}

	fn readonly(&self) -> bool {
		self.root
			.metadata()
			.map(|metadata| metadata.permissions().readonly())
			.unwrap_or(false)
	}

	fn iter<'a>(&'a self) -> Result<Box<dyn Iterator<Item = Name> + 'a>> {
		let mut walk_dir = WalkDir::new(&self.root).min_depth(1);
		if !self.recursive {
			walk_dir = walk_dir.max_depth(1);
		}
		Ok(Box::new(
			walk_dir
				.follow_links(true)
				.into_iter()
				.filter_map(move |entry| {
					let entry = match entry {
						Ok(entry) => entry,
						Err(err) => {
							warn!("Could not read file system entry: {}", err);
							return None;
						}
					};

					match entry.metadata() {
						Ok(metadata) => {
							if !metadata.is_file() {
								return None;
							}
						}
						Err(err) => {
							error!(
								"Could not read metadata for \"{}\": {}",
								entry.path().to_string_lossy(),
								err
							);
							return None;
						}
					}

					match entry.path().strip_prefix(&self.root) {
						Ok(path) if cfg!(windows) => {
							// Use forward slashes instead of backslashes
							let mut components = path.components().peekable();
							let mut buf = String::new();
							while let Some(component) = components.next() {
								match component {
									Component::Normal(path) => {
										buf.push_str(&path.to_string_lossy());
										if components.peek().is_some() {
											buf.push('/');
										} else {
											break;
										}
									}
									_ => unreachable!(),
								}
							}

							Some(buf.into())
						}
						Ok(path) => Some(path.to_string_lossy().into()),
						Err(err) => {
							warn!("Could not strip file system path: {}", err);
							None
						}
					}
				}),
		))
	}

	fn create(&mut self, path: Name) -> Result<Box<dyn Write>> {
		let path = self.concat_path(path);
		Ok(Box::new(
			OpenOptions::new()
				.write(true)
				.create(true)
				.open(&path)
				.chain_with(ErrorKind::Io, || {
					format!("Could not create file {:?}", path)
				})?,
		))
	}

	fn create_new(&mut self, path: Name) -> Result<Box<dyn Write>> {
		let path = self.concat_path(path);
		Ok(Box::new(
			OpenOptions::new()
				.write(true)
				.create_new(true)
				.open(&path)
				.chain_with(ErrorKind::Io, || {
					format!("Could not create file new {:?}", path)
				})?,
		))
	}

	fn exists(&self, path: Name) -> bool {
		self.concat_path(path).exists()
	}

	fn modified(&self, path: Name) -> Result<SystemTime> {
		Ok(self
			.concat_path(path)
			.metadata()
			.context(ErrorKind::Io)?
			.modified()
			.context(ErrorKind::Io)?)
	}

	fn open(&self, path: Name) -> Result<Box<dyn Read>> {
		let path = self.concat_path(path);
		Ok(Box::new(
			OpenOptions::new()
				.read(true)
				.open(&path)
				.chain_with(ErrorKind::Io, || {
					format!("Could not open path {:?}", path)
				})?,
		))
	}

	fn remove(&mut self, path: Name) -> Result<()> {
		let path = self.concat_path(path);
		fs::remove_file(&path).chain_with(ErrorKind::Io, || {
			format!("Could not open path {:?}", path)
		})
	}
}
