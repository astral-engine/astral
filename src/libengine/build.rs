// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use rustc_version::Channel;

fn main() {
	println!("cargo:rerun-if-changed=build.rs");

	if let Ok(meta) = rustc_version::version_meta() {
		if let Channel::Nightly = meta.channel {
			println!("cargo:rustc-cfg=unstable");
		}
	}
}
