// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, December 2018

use slog::{info, o, Logger};

pub struct System {
	log: Logger,
}

impl System {
	pub fn new(engine: &astral_engine::Engine) -> Self {
		let log = engine.logger().new(o!("system" => "video"));
		info!(log, "initializing"; "version" => env!("CARGO_PKG_VERSION"));
		Self { log }
	}

	pub fn logger(&self) -> &Logger {
		&self.log
	}
}

impl Drop for System {
	fn drop(&mut self) {
		info!(self.logger(), "shutting down")
	}
}
