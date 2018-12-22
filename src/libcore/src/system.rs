// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, December 2018

use std::fmt::{self, Debug, Formatter};

use astral_engine::third_party::slog::{info, o, Logger};

/// Core system for the Astral Engine.
///
/// See the [module-level documentation] for more.
///
/// [module-level documentation]: index.html
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
	/// # use astral::third_party::slog;
	///
	/// # let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	let engine = Engine::new(&logger);
	/// # #[allow(unused_variables)]
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
	/// use astral::{
	/// 	core,
	/// 	third_party::slog::info,
	/// };
	/// # use astral::third_party::slog;
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

impl Debug for System {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		fmt.debug_struct("System").finish()
	}
}

impl Drop for System {
	fn drop(&mut self) {
		info!(self.logger(), "shutting down")
	}
}
