// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc, Weak,
};

use crate::LoadPriority;

#[derive(Debug)]
pub struct Declaration {
	load_priority: LoadPriority,
	parents: Vec<Weak<Declaration>>,
	children: Vec<Arc<Declaration>>,
	completed: AtomicBool,
	canceled: AtomicBool,
	stalled: AtomicBool,
	released: AtomicBool,
}

impl Declaration {
	pub fn new() -> Self {
		Self {
			load_priority: LoadPriority::default(),
			parents: Vec::default(),
			children: Vec::default(),
			completed: AtomicBool::new(false),
			canceled: AtomicBool::new(false),
			stalled: AtomicBool::new(false),
			released: AtomicBool::new(false),
		}
	}

	pub fn with_priority(load_priority: LoadPriority) -> Self {
		Self {
			load_priority,
			..Self::default()
		}
	}
	pub fn completed(&self) -> bool {
		self.completed.load(Ordering::Relaxed)
	}

	pub fn canceled(&self) -> bool {
		self.canceled.load(Ordering::Relaxed)
	}

	pub fn loading(&self) -> bool {
		!self.completed() && !self.canceled()
	}

	pub fn active(&self) -> bool {
		!self.canceled() && !self.released()
	}

	pub fn stalled(&self) -> bool {
		self.stalled.load(Ordering::Relaxed)
	}

	pub fn released(&self) -> bool {
		self.released.load(Ordering::Relaxed)
	}

	pub fn cancel(&self) {
		if self.released() || self.canceled() {
			return;
		}

		unimplemented!()
	}

	pub fn release(&self) {
		if self.released() || self.canceled() {
			return;
		}

		unimplemented!()
	}

	pub fn stall(&self) {
		self.stalled.store(true, Ordering::Relaxed)
	}

	pub fn start_stalled(&self) {
		if !self.active() {
			return;
		}

		if !self
			.stalled
			.compare_and_swap(true, false, Ordering::Relaxed)
		{
			// return if not stalled
			return;
		}

		unimplemented!("manager.start()")
	}
}

impl Default for Declaration {
	fn default() -> Self {
		Self::new()
	}
}
