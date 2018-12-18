// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, December 2018

use crate::System;

use astral_engine::third_party::slog::{info, o, Logger};

pub struct Subsystem {
	log: Logger,
}

impl Subsystem {
	pub fn new(system: &System) -> Self {
		let log = system.logger().new(o!("subsystem" => "string"));
		info!(log, "initializing");
		Self { log }
	}

	pub fn logger(&self) -> &Logger {
		&self.log
	}
}

impl Drop for Subsystem {
	fn drop(&mut self) {
		info!(self.logger(), "shutting down")
	}
}
