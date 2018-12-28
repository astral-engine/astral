// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

#![doc(
	html_no_source,
	html_logo_url = "https://astral-engine.github.io/docs/logo_astral.svg",
	html_favicon_url = "https://astral-engine.github.io/docs/logo.svg"
)]

#[doc(inline)]
pub use {
	astral_core as core,
	astral_engine::*,
	astral_resource as resource,
	astral_video as video,
};

pub mod prelude {
	pub use astral_engine::third_party::rayon::prelude::*;
}
