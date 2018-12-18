// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, December 2018

use astral_engine::third_party::slog::{info, o, Logger};

pub struct System {
	log: Logger,
}

impl System {
	/// Initialize the core system from the given [`Engine`].
	///
	/// # Example
	///
	/// ```
	/// use astral::{core, Engine};
	///
	/// # let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	let engine = Engine::new(&logger);
	/// let core_system = core::System::new(&engine);
	/// ```
	pub fn new(engine: &astral_engine::Engine) -> Self {
		let log = engine.logger().new(o!("system" => "core"));
		info!(log, "initializing"; "version" => env!("CARGO_PKG_VERSION"));
		Self { log }
	}

	/// Returns the logger of this core system.
	///
	/// # Example
	///
	/// ```
	/// use slog::info;
	///
	/// use astral::core;
	///
	/// # let logger = slog::Logger::root(slog::Discard, slog::o!());
	/// # let engine = astral::Engine::new(&logger);
	/// let core_system = core::System::new(&engine);
	///
	/// info!(core_system.logger(), "foo bar"; "additional" => "information");
	/// ```
	pub fn logger(&self) -> &Logger {
		&self.log
	}
}

impl Drop for System {
	fn drop(&mut self) {
		info!(self.logger(), "shutting down")
	}
}
