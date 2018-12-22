// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::{
	boxed::Box,
	fs::{self, OpenOptions},
	hash::{BuildHasher, BuildHasherDefault},
	io::{Read, Write},
	path::{Component, PathBuf},
	time::SystemTime,
};

use astral_core::{
	error::ResultExt,
	hash::Murmur3,
	string::{self, Name},
};
use astral_engine::third_party::{
	slog::{error, o, warn, Logger},
	walkdir::WalkDir,
};

use super::{ErrorKind, Result, Subsystem, VirtualFileSystem};

/// A `FileSystem` is a view into the systems file system.
#[derive(Debug)]
pub struct FileSystem<'str, 'ass, H = BuildHasherDefault<Murmur3>> {
	logger: Logger,
	root: PathBuf,
	recursive: bool,
	string_subsystem: &'str string::Subsystem<H>,
	asset_subsystem: &'ass Subsystem,
}

impl<'str, 'ass, H> FileSystem<'str, 'ass, H> {
	/// Construct a new `FileSystem` at the specified root path.
	/// If wished, the file system can search files in a recursive manner.
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
	/// use astral::{
	/// 	core::string::Name,
	/// 	resource::assets::{FileSystem, VirtualFileSystem},
	/// };
	///
	/// let file_system = FileSystem::new(".", &asset_subsystem, &string_subsystem)?;
	/// let file_name = Name::new("a.txt", &string_subsystem);
	/// # #[allow(unused_variables)]
	/// let file = file_system.open(file_name.into())?;
	/// # Ok(())
	/// # }
	/// ```
	#[allow(clippy::new_ret_no_self)]
	pub fn new<P: Into<PathBuf>>(
		root: P,
		asset_subsystem: &'ass Subsystem,
		string_subsystem: &'str string::Subsystem<H>,
	) -> Result<Self> {
		let root = root.into();
		Ok(Self {
			logger: asset_subsystem
				.logger()
				.new(o!("file system" => root.to_string_lossy().to_string())),
			root,
			recursive: true,
			string_subsystem,
			asset_subsystem,
		})
	}

	/// test func
	pub fn test() -> Self {
		unimplemented!()
	}

	fn concat_path(&self, path: Name<'str, H>) -> PathBuf {
		let mut path_buf = self.root.clone();
		path_buf.push(path.to_string());
		path_buf
	}
}

impl<'str, H> VirtualFileSystem<'str, H> for FileSystem<'str, '_, H>
where
	H: BuildHasher + std::fmt::Debug + Send + Sync,
{
	fn name(&self) -> Name<'str, H> {
		Name::new(self.root.to_string_lossy(), self.string_subsystem)
	}

	fn readonly(&self) -> bool {
		self.root
			.metadata()
			.map(|metadata| metadata.permissions().readonly())
			.unwrap_or(false)
	}

	fn iter<'a>(&'a self) -> Result<Box<dyn Iterator<Item = Name<'str, H>> + 'a>> {
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
							warn!(
								self.logger,
								"Could not read file system entry";
								"error" => &err.to_string()
							);
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
								self.logger,
								"Could not read metadata";
								"path" => %entry.path().to_string_lossy(),
								"error" => &err.to_string()
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

							Some(Name::new(&buf, self.string_subsystem))
						}
						Ok(path) => Some(Name::new(path.to_string_lossy(), self.string_subsystem)),
						Err(err) => {
							warn!(
								self.logger,
								"Could not strip file system path";
								"error" => &err.to_string()
							);
							None
						}
					}
				}),
		))
	}

	fn create(&mut self, path: Name<'str, H>) -> Result<Box<dyn Write>> {
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

	fn create_new(&mut self, path: Name<'str, H>) -> Result<Box<dyn Write>> {
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

	fn exists(&self, path: Name<'str, H>) -> bool {
		self.concat_path(path).exists()
	}

	fn modified(&self, path: Name<'str, H>) -> Result<SystemTime> {
		Ok(self
			.concat_path(path)
			.metadata()
			.context(ErrorKind::Io)?
			.modified()
			.context(ErrorKind::Io)?)
	}

	fn open(&self, path: Name<'str, H>) -> Result<Box<dyn Read>> {
		let path = self.concat_path(path);
		Ok(Box::new(
			OpenOptions::new()
				.read(true)
				.open(&path)
				.chain_with(ErrorKind::Io, || format!("Could not open path {:?}", path))?,
		))
	}

	fn remove(&mut self, path: Name<'str, H>) -> Result<()> {
		let path = self.concat_path(path);
		fs::remove_file(&path)
			.chain_with(ErrorKind::Io, || format!("Could not open path {:?}", path))
	}
}
