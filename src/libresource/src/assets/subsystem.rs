// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, December 2018

use crate::System;

use std::fmt::{self, Debug, Formatter};

use astral_engine::third_party::slog::{info, o, Logger};

pub struct Subsystem {
	log: Logger,
}

impl Subsystem {
	/// Initialize the asset subsystem from the given [resource system].
	///
	/// [resource system]: astral_resource::System
	///
	/// # Example
	///
	/// ```
	/// use astral::{
	/// 	Engine,
	/// 	resource::{self, assets},
	/// };
	/// # use astral::third_party::slog;
	///
	/// # let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	let engine = Engine::new(&logger);
	/// let resource_system = resource::System::new(&engine);
	/// # #[allow(unused_variables)]
	/// let asset_subsystem = assets::Subsystem::new(&resource_system);
	/// ```
	pub fn new(system: &System) -> Self {
		let log = system.logger().new(o!("subsystem" => "assets"));
		info!(log, "initializing");
		Self { log }
	}

	/// Returns the logger of this asset subsystem.
	///
	/// # Example
	///
	/// ```
	/// use astral::{
	/// 	Engine,
	/// 	resource::{self, assets},
	/// 	third_party::slog::info,
	/// };
	/// # use astral::third_party::slog;
	///
	/// let logger = slog::Logger::root(slog::Discard, slog::o!());
	/// let engine = Engine::new(&logger);
	/// let resource_system = resource::System::new(&engine);
	/// let asset_subsystem = assets::Subsystem::new(&resource_system);
	///
	/// info!(asset_subsystem.logger(), "foo bar"; "additional" => "information");
	/// ```
	pub fn logger(&self) -> &Logger {
		&self.log
	}
}

impl Debug for Subsystem {
	fn fmt(&self, fmt: &mut Formatter<'_>) -> fmt::Result {
		fmt.debug_struct("Subsystem").finish()
	}
}

impl Drop for Subsystem {
	fn drop(&mut self) {
		info!(self.logger(), "shutting down")
	}
}
