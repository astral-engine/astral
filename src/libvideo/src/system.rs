// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, December 2018

use astral_engine::third_party::slog::{info, o, Logger};

pub struct System {
	log: Logger,
}

impl System {
	/// Initialize the video system from the given [`Engine`].
	///
	/// # Example
	///
	/// ```
	/// use astral::{video, Engine};
	/// # use astral::third_party::slog;
	///
	/// # let logger = slog::Logger::root(slog::Discard, slog::o!());
	///	let engine = Engine::new(&logger);
	/// let video_system = video::System::new(&engine);
	/// ```
	pub fn new(engine: &astral_engine::Engine) -> Self {
		let log = engine.logger().new(o!("system" => "video"));
		info!(log, "initializing"; "version" => env!("CARGO_PKG_VERSION"));
		Self { log }
	}

	/// Returns the logger of this video system.
	///
	/// # Example
	///
	/// ```
	/// use astral::{
	/// 	third_party::slog::info,
	/// 	video,
	/// };
	/// # use astral::third_party::slog;
	///
	/// # let logger = slog::Logger::root(slog::Discard, slog::o!());
	/// # let engine = astral::Engine::new(&logger);
	/// let video_system = video::System::new(&engine);
	///
	/// info!(video_system.logger(), "foo bar"; "additional" => "information");
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
