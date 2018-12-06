// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use log::{LevelFilter, Log, Metadata, Record};

/// Provides a stderr/out based Logger implementation.
///
/// # Example
///
/// ```rust,no_run
/// use astral::core::log::{self, TerminalLogger, LevelFilter, info};
///
/// # fn main() -> Result<(), log::SetLoggerError> {
/// let logger = Box::new(TerminalLogger::default());
/// log::set_max_level(LevelFilter::Trace);
/// log::set_boxed_logger(logger)?;
///
/// info!("log a message: {}", 42);
/// # Ok(()) }
/// ```
pub struct TerminalLogger(Box<dyn Log>);

impl Default for TerminalLogger {
	fn default() -> Self {
		Self::new()
	}
}

impl TerminalLogger {
	/// Creates a new `TerminalLogger`
	pub fn new() -> Self {
		TerminalLogger(
			if let Some(logger) =
				simplelog::TermLogger::new(LevelFilter::max(), simplelog::Config::default())
			{
				logger
			} else {
				simplelog::SimpleLogger::new(LevelFilter::max(), simplelog::Config::default())
			},
		)
	}
}

impl Log for TerminalLogger {
	fn enabled(&self, metadata: &Metadata<'_>) -> bool {
		self.0.enabled(metadata)
	}

	fn log(&self, record: &Record<'_>) {
		self.0.log(record)
	}

	fn flush(&self) {
		self.0.flush()
	}
}
