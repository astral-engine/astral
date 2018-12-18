// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, December 2018

use std::fmt::{self, Debug, Formatter};

use slog::{info, o, Logger};

/// The main object for handling systems in the Astral Engine.
pub struct Engine {
	log: Logger,
}

impl Engine {
	/// Initialize the engine with the given [`Logger`].
	///
	/// For more information on the logger see the [slog] crate.
	///
	/// [`Logger`]: third_party::slog::Logger
	/// [slog]: third_party::slog
	///
	/// # Example
	///
	/// ```
	/// use slog_async::Async;
	/// use slog_term::{CompactFormat, PlainDecorator};
	///
	/// use astral::{
	/// 	Engine,
	/// 	third_party::slog::{Drain, Logger, o},
	/// };
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

	/// Returns the logger of this engine.
	///
	/// # Example
	///
	/// ```
	/// use astral::{
	///     Engine,
	///     third_party::slog::info,
	/// };
	/// # use astral::third_party::slog;
	///
	/// # let logger = slog::Logger::root(slog::Discard, slog::o!());
	/// let engine = Engine::new(&logger);
	///
	/// info!(engine.logger(), "foo bar"; "additional" => "information");
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
