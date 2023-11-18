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
//!         .add_plugins(SpriteSheetLoaderPlugin)
//!         .add_systems(Startup, load_spritesheet)
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
    asset::{io::Reader, AssetIndex, AssetLoader, AssetPath, AsyncReadExt, LoadContext},
    prelude::{App, AssetApp, Handle, Image, Plugin, Vec2},
    reflect::{FromReflect, Reflect},
    render::render_resource::Texture,
    sprite::{TextureAtlas, TextureAtlasBuilder},
    utils::BoxedFuture,
};
use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

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

/// Possible errors that can be produced by [`SpriteSheetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SpriteSheetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}

/// File extension for spritesheet manifest files written in ron.
pub const FILE_EXTENSIONS: &[&str] = &["titan"];

impl AssetLoader for SpriteSheetLoader {
    type Asset = TextureAtlas;
    type Settings = ();
    type Error = SpriteSheetLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let titan_entries = ron::de::from_bytes::<Vec<TitanEntry>>(&bytes)?;

            /* TODO: Actually consider others */
            let entry = &titan_entries[0];

            // Collect all images and create a new one
            let image_asset_path = AssetPath::from_path(Path::new(&entry.path));
            let image = load_context
                .load_direct(image_asset_path.clone())
                .await
                .unwrap();
            let image = image.take::<Image>().unwrap();
            let image_size = image.size_f32();

            // Create a Handle from the Image
            let image_handle = load_context.add_loaded_labeled_asset("image", image.into());

            let mut texture_atlas = TextureAtlas::new_empty(image_handle, image_size);
            /* TODO: Handle other options */
            if let TitanSpriteSheet::Homogeneous{tile_size, columns, rows, ..} = &entry.sprite_sheet {
                for i in 0..*rows {
                    for j in 0..*columns {
                        /* TODO: Padding and offest */
                        let rect = bevy::math::Rect {
                            min: Vec2::new(j as f32 * tile_size.x, i as f32 * tile_size.y),
                            max: Vec2::new(
                                (j + 1) as f32 * tile_size.x,
                                (i + 1) as f32 * tile_size.y,
                            ),
                        };
                        texture_atlas.add_texture(rect);
                    }
                }
            }

            Ok(texture_atlas)
        })
    }

    fn extensions(&self) -> &[&str] {
        FILE_EXTENSIONS
    }
}

/* TODO: Parse a Vec of this */
#[derive(Debug, Deserialize)]
struct TitanEntry {
    path: String,
    #[serde(default)]
    sprite_sheet: TitanSpriteSheet,
}

#[derive(Debug, Default, Deserialize)]
enum TitanSpriteSheet {
    #[default]
    None,
    Homogeneous{
        tile_size: Vec2,
        columns: usize,
        rows: usize,
        #[serde(default)]
        padding: Option<Vec2>,
        #[serde(default)]
        offset: Option<Vec2>,
    },
    Heterogeneous(Vec<bevy::math::Rect>),
}
