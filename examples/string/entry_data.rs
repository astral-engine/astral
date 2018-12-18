// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, December 2018

use std::{num::NonZeroU32, sync::atomic::AtomicPtr};

pub struct EntryData {
	pub(super) next: AtomicPtr<EntryData>,
	pub(super) index: Option<NonZeroU32>,
}
