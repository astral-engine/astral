// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, December 2018

use std::{num::NonZeroU32, sync::atomic::AtomicUsize};

use super::EntryData;

pub struct Entry<'a> {
	pub(super) data: EntryData,
	pub(super) string: &'a str,
}
