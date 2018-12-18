// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, December 2018

//! Entry point for creating the [`Engine`] object.

#![doc(
	html_no_source,
	html_logo_url = "https://astral-engine.github.io/docs/logo_astral.svg",
	html_favicon_url = "https://astral-engine.github.io/docs/logo.svg"
)]
#![warn(
	future_incompatible,
	nonstandard_style,
	rust_2018_compatibility,
	rust_2018_idioms,
	unused,
	box_pointers,
	macro_use_extern_crate,
	missing_copy_implementations,
	missing_debug_implementations,
	missing_docs,
	missing_doc_code_examples,
	single_use_lifetimes,
	trivial_casts,
	trivial_numeric_casts,
	unreachable_pub,
	unsafe_code,
	unused_import_braces,
	unused_lifetimes,
	unused_qualifications,
	unused_results,
	variant_size_differences,
	clippy::pedantic
)]

use std::fmt::{self, Debug, Formatter};

use slog::{info, o, Logger};

/// The main object for handling systems in the Astral Engine.
pub struct Engine {
	log: Logger,
}

impl Engine {
	/// Initialize the engine with the given Logger.
	///
	/// # Example
	///
	/// ```
	/// use slog::{Drain, Logger, o};
	/// use slog_async::Async;
	/// use slog_term::{CompactFormat, PlainDecorator};
	///
	/// use astral::Engine;
	///
	/// let decorator = PlainDecorator::new(std::io::stdout());
	/// let drain = CompactFormat::new(decorator).build().fuse();
	/// let drain = Async::new(drain).build().fuse();
	///
	/// let log = Logger::root(drain, o!());
	///
	/// let engine = Engine::new(&log);
	/// ```
	pub fn new(logging_root: &Logger) -> Self {
		let log = logging_root.new(o!());
		info!(log, "initializing Astral Engine"; "version" => env!("CARGO_PKG_VERSION"));
		Self { log }
	}

	/// Returns the logger of the engine.
	///
	/// # Example
	///
	/// ```
	/// use slog::info;
	///
	/// use astral::Engine;
	///
	/// # let logger = slog::Logger::root(slog::Discard, slog::o!());
	/// let engine = Engine::new(&logger);
	///
	/// info!(engine.logger(), "foo bar");
	/// ```
	pub fn logger(&self) -> &Logger {
		&self.log
	}
}

impl Drop for Engine {
	fn drop(&mut self) {
		info!(self.logger(), "shutting down")
	}
}

impl Debug for Engine {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		fmt.debug_struct("Engine").finish()
	}
}
