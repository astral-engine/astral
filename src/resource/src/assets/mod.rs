// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

//! Functionality to manage asset data.
//!
//! Asset data is used for creating a resource, which depends on external
//! content. To uniquely identify asset data, a [`Location`] is used. Each
//! [`Location`] contains a [`NamespaceId`] and a [`Name`]. With the
//! [`NamespaceId`], the [`Namespace`] can be retrieved from a [`Catalog`].
//! The [`Name`] identifies an asset path in a [`Namespace`]. For convenience,
//! the [`Catalog`] provides the same methods as a [`Namespace`], but with a
//! [`Location`] as parameter instead of a [`Name`].
//!
//! Each [`Namespace`] consists of multiple [`VirtualFileSystem`]s. Each one is
//! an abstract view into an abstract file system like [`FileSystem`]. It
//! contains methods for creating, removing and opening file locations.
//!
//! [`Location`]: struct.Location.html
//! [`NamespaceId`]: struct.NamespaceId.html
//! [`Namespace`]: struct.Namespace.html
//! [`Catalog`]: struct.Catalog.html
//! [`Name`]: ../../core/string/struct.Name.html
//! [`FileSystem`]: struct.FileSystem.html
//! [`VirtualFileSystem`]: trait.VirtualFileSystem.html

mod catalog;
mod error;
mod file_system;
mod location;
mod namespace;
mod namespace_id;
mod virtual_file_system;
mod virtual_file_system_index;

pub use self::{
	catalog::Catalog,
	error::{Error, ErrorKind, Result},
	file_system::FileSystem,
	location::Location,
	namespace::Namespace,
	namespace_id::NamespaceId,
	virtual_file_system::VirtualFileSystem,
	virtual_file_system_index::VirtualFileSystemIndex,
};
