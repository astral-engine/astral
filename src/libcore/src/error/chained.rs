// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::error;

#[derive(Debug)]
pub(super) struct Chained<Kind> {
	pub(super) kind: Kind,
	pub(super) error: Box<dyn error::Error + Send + Sync>,
	pub(super) source: Box<dyn error::Error + Send + Sync>,
}
