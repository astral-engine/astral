// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, December 2018

use slog::{info, o, Logger};

pub struct Engine {
	log: Logger,
}

impl Engine {
	pub fn new(logging_root: &Logger) -> Self {
		let log = logging_root.new(o!());
		info!(log, "initializing Astral Engine"; "version" => env!("CARGO_PKG_VERSION"));
		Self { log }
	}

	pub fn logger(&self) -> &Logger {
		&self.log
	}
}

impl Drop for Engine {
	fn drop(&mut self) {
		info!(self.logger(), "shutting down")
	}
}
