// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, December 2018

use crate::System;

use astral_engine::third_party::slog::{info, o, Logger};

/// Manages optimized string allocation.
///
/// See the [module-level documentation] for more.
///
/// [module-level documentation]: index.html
pub struct Subsystem {
	log: Logger,
}

impl Subsystem {
	/// Initialize the string subsystem from the given [core system].
	///
	/// [core system]: astral_core::System
	///
	/// # Example
	///
	/// ```
	/// use astral::{
	/// 	Engine,
	/// 	core::{self, string},
	/// };
	/// # use astral::third_party::slog;
	///
	/// # let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	let engine = Engine::new(&logger);
	/// let core_system = core::System::new(&engine);
	/// let asset_subsystem = string::Subsystem::new(&core_system);
	/// ```
	pub fn new(system: &System) -> Self {
		let log = system.logger().new(o!("subsystem" => "string"));
		info!(log, "initializing");
		Self { log }
	}

	/// Returns the logger of this string subsystem.
	///
	/// # Example
	///
	/// ```
	/// use astral::{
	/// 	Engine,
	/// 	core::{self, string},
	/// 	third_party::slog::info,
	/// };
	/// # use astral::third_party::slog;
	///
	/// # let logger = slog::Logger::root(slog::Discard, slog::o!());
	/// # let engine = astral::Engine::new(&logger);
	/// # let core_system = core::System::new(&engine);
	/// let asset_subsystem = string::Subsystem::new(&core_system);
	///
	/// info!(asset_subsystem.logger(), "foo bar"; "additional" => "information");
	/// ```
	pub fn logger(&self) -> &Logger {
		&self.log
	}
}

impl Drop for Subsystem {
	fn drop(&mut self) {
		info!(self.logger(), "shutting down")
	}
}
