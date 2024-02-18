#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(unused_imports, missing_docs)]

use bevy::{
    asset::AssetApp,
    prelude::{App, Plugin},
};

pub mod asset_loader;
mod serde;

/// Adds support for spritesheet manifest files loading to the app.
pub struct SpriteSheetLoaderPlugin;

impl Plugin for SpriteSheetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<asset_loader::TextureAtlas>()
            .init_asset_loader::<asset_loader::SpriteSheetLoader>();
    }
}

/// `use bevy_titan::prelude::*;` to import common components and plugins.
pub mod prelude {
    pub use crate::asset_loader::SpriteSheetLoaderError;
    pub use crate::asset_loader::TextureAtlas;
    pub use crate::SpriteSheetLoaderPlugin;
}
