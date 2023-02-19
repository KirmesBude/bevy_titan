//! This crate allows you to directly load a TextureAtlas from a manifest file.
//!
//! `bevy_titan` introduces a [`SpriteSheetManifest`](crate::SpriteSheetManifest) and the corresponding [`SpriteSheetLoader`](crate::SpriteSheetLoader).
//! Assets with the 'titan' extension can be loaded just like any other asset via the [`AssetServer`](::bevy::asset::AssetServer)
//! and will yield a [`TextureAtlas`](::bevy::sprite::TextureAtlas) [`Handle`](::bevy::asset::Handle).
//!
//! ### `spritesheet.titan`
//! ```rust,ignore
//! SpriteSheetManifest ( /* The explicit type name can be omitted */
//!     path: String, /* path to spritesheet image asset */
//!     tile_size: (
//!         w: f32,
//!         h: f32,
//!     ),
//!     columns: usize,
//!     rows: usize,
//!    // These can be optionally defined
//!    /*
//!    padding: (
//!        h: f32,
//!        w: f32,
//!    ),
//!    offset: (
//!        h: f32,
//!        w: f32,
//!    ),
//!    */
//! )
//! ```
//!
//! ```edition2021
//! # use bevy_titan::SpriteSheetLoaderPlugin;
//! # use bevy::prelude::*;
//! #
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugin(SpriteSheetLoaderPlugin)
//!         .add_system(load_spritesheet)
//!         .run();
//! }
//!
//! fn load_spritesheet(mut commands: Commands, asset_server: Res<AssetServer>) {
//!     let texture_atlas_handle = asset_server.load("spritesheet.titan");
//!     commands.spawn(Camera2dBundle::default());
//!     commands.spawn(
//!         SpriteSheetBundle {
//!              texture_atlas: texture_atlas_handle,
//!              transform: Transform::from_scale(Vec3::splat(6.0)),
//!              ..default()
//!         }
//!     );
//! }
//!
//! ```

#![forbid(unsafe_code)]
#![warn(unused_imports, missing_docs)]

use bevy::{
    asset::{AssetLoader, AssetPath, LoadContext, LoadedAsset},
    prelude::{AddAsset, App, Handle, Image, Plugin, Vec2},
    sprite::TextureAtlas,
    utils::BoxedFuture,
};
use serde::Deserialize;
use std::path::Path;

/// Adds support for spritesheet manifest files loading to the app.
pub struct SpriteSheetLoaderPlugin;

impl Plugin for SpriteSheetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<SpriteSheetLoader>();
    }
}

/// Loader for spritesheet manifest files written in ron. Loads a TextureAtlas asset.
#[derive(Default)]
pub struct SpriteSheetLoader;

/// File extension for spritesheet manifest files written in ron.
pub const FILE_EXTENSIONS: &[&str] = &["titan"];

impl AssetLoader for SpriteSheetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let spritesheet_manifest = ron::de::from_bytes::<SpriteSheetManifest>(bytes)?;

            let image_asset_path = AssetPath::new_ref(Path::new(&spritesheet_manifest.path), None);
            let image_handle: Handle<Image> = load_context.get_handle(image_asset_path.clone());

            let texture_atlas = TextureAtlas::from_grid(
                image_handle,
                spritesheet_manifest.tile_size.into(),
                spritesheet_manifest.columns,
                spritesheet_manifest.rows,
                spritesheet_manifest.padding.map(|x| x.into()),
                spritesheet_manifest.offset.map(|x| x.into()),
            );

            let atlas_asset = LoadedAsset::new(texture_atlas).with_dependency(image_asset_path);

            load_context.set_default_asset(atlas_asset);
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        FILE_EXTENSIONS
    }
}

/// Declaration of the deserialized struct from the spritesheet manifest file written in ron.
/// Note: This is only public for the purpose to document the ron/titan format.
#[derive(Debug, Deserialize)]
pub struct SpriteSheetManifest {
    /// Path to the spritesheet image asset.
    pub path: String,
    /// Width and height of a tile inside the spritesheet.
    pub tile_size: Rect,
    /// How many columns of tiles there are inside the spritesheet.
    pub columns: usize,
    /// How many rows of tiles there are inside the spritesheet.
    pub rows: usize,
    #[serde(default)]
    /// Padding between tiles.
    pub padding: Option<Rect>,
    #[serde(default)]
    /// Offset from the top left from where the tiling begins.
    pub offset: Option<Rect>,
}

/// Helper struct to represent Vec2.
/// Note: This is only public for the purpose to document the ron/titan format.
#[derive(Debug, Deserialize)]
pub struct Rect {
    w: f32,
    h: f32,
}

impl From<Rect> for Vec2 {
    fn from(value: Rect) -> Self {
        Self::new(value.w, value.h)
    }
}
