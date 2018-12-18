// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, December 2018

use astral_engine::third_party::slog::{info, o, Logger};

pub struct System {
	log: Logger,
}

impl System {
	/// Initialize the resource system from the given [`Engine`].
	///
	/// # Example
	///
	/// ```
	/// use astral::{resource, Engine};
	///
	/// # let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	let engine = Engine::new(&logger);
	/// let resource_system = resource::System::new(&engine);
	/// ```
	pub fn new(engine: &astral_engine::Engine) -> Self {
		let log = engine.logger().new(o!("system" => "resource"));
		info!(log, "initializing"; "version" => env!("CARGO_PKG_VERSION"));
		Self { log }
	}

	/// Returns the logger of this resource system.
	///
	/// # Example
	///
	/// ```
	/// use slog::info;
	///
	/// use astral::resource;
	///
	/// # let logger = slog::Logger::root(slog::Discard, slog::o!());
	/// # let engine = astral::Engine::new(&logger);
	/// let resource_system = resource::System::new(&engine);
	///
	/// info!(resource_system.logger(), "foo bar"; "additional" => "information");
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
