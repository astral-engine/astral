// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

//! Structures and macros for logging.
//!
//! The `logging` module provides the logging API that abstracts over the
//! actual logging implementation.
//!
//! If no logging implementation is selected, the facade falls back to a "noop"
//! implementation that ignores all log messages. The overhead in this case
//! is very small - just an integer load, comparison and jump.
//!
//! A log request consists of a _target_, a _level_, and a _body_. A target is a
//! string which defaults to the module path of the location of the log request,
//! though that default may be overridden. Logger implementations typically use
//! the target to filter requests based on some user configuration.
//!
//! For more information see the [log crate].
//!
//! [log crate]: https://docs.rs/log
//!
//! # Use
//!
//! The basic use of the log crate is through the five logging macros:
//! [`error!`], [`warn!`], [`info!`], [`debug!`] and [`trace!`]
//! where `error!` represents the highest-priority log messages
//! and `trace!` the lowest. The log messages are filtered by configuring
//! the log level to exclude messages with a lower priority.
//! Each of these macros accept format strings similarly to [`println!`].
//!
//! ### Example
//!
//! ```rust
//! # extern crate astral;
//! use astral::core::log::{info, warn};
//!
//! # #[derive(Debug)] pub struct Yak(String);
//! # impl Yak { fn shave(&mut self, _: u32) {} }
//! # fn find_a_razor() -> Result<u32, u32> { Ok(1) }
//! pub fn shave_the_yak(yak: &mut Yak) {
//!     info!(target: "yak_events", "Commencing yak shaving for {:?}", yak);
//!
//!     loop {
//!         match find_a_razor() {
//!             Ok(razor) => {
//!                 info!("Razor located: {}", razor);
//!                 yak.shave(razor);
//!                 break;
//!             }
//!             Err(err) => {
//!                 warn!("Unable to locate a razor: {}, retrying", err);
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! [`error!`]: ./macro.error.html
//! [`warn!`]: ./macro.warn.html
//! [`info!`]: ./macro.info.html
//! [`debug!`]: ./macro.debug.html
//! [`trace!`]: ./macro.trace.html
//! [`println!`]: https://doc.rust-lang.org/stable/std/macro.println.html

extern crate log;

mod terminal_logger;

pub use self::{log::*, terminal_logger::TerminalLogger};
