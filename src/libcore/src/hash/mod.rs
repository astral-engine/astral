// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

//! Hashing utilities and hashers.

mod murmur3;
mod nop_hasher;

pub use self::{murmur3::Murmur3, nop_hasher::NopHasher};
