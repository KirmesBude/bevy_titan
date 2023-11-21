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
    asset::{io::Reader, AssetLoader, AssetPath, AsyncReadExt, LoadContext, LoadDirectError},
    prelude::{App, AssetApp, Image, Plugin, Rect, UVec2, Vec2},
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::TextureFormatPixelInfo,
    },
    sprite::TextureAtlas,
    utils::BoxedFuture,
};
use rectangle_pack::{
    contains_smallest_box, pack_rects, volume_heuristic, GroupedRectsToPlace, PackedLocation,
    RectToInsert, TargetBin,
};
use serde::Deserialize;
use std::{collections::BTreeMap, path::Path};
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
    /// A LoadDirect Error
    #[error("Could not load: {0}")]
    LoadDirectError(#[from] LoadDirectError),
    /// A NotAnImage Error
    #[error("Loading from {0} does not provide Image")]
    NotAnImage(String),
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

            /* Save rect ids and images for later use */
            let rect_ids_len = titan_entries.iter().fold(0, |acc, titan_entry| {
                acc + match &titan_entry.sprite_sheet {
                    TitanSpriteSheet::None => 1,
                    TitanSpriteSheet::Homogeneous { columns, rows, .. } => columns * rows,
                    TitanSpriteSheet::Heterogeneous(vec) => vec.len(),
                }
            });
            let mut rect_ids = Vec::with_capacity(rect_ids_len);
            let images_len = titan_entries.len();
            let mut images = Vec::with_capacity(images_len);
            for (titan_entry_index, titan_entry) in titan_entries.into_iter().enumerate() {
                /* Load the image */
                let image_asset_path = AssetPath::from_path(Path::new(&titan_entry.path));
                let image = load_context.load_direct(image_asset_path).await?;
                let image = image
                    .take::<Image>()
                    .ok_or(SpriteSheetLoaderError::NotAnImage(titan_entry.path.clone()))?;

                /* Get and insert all rects */
                match titan_entry.sprite_sheet {
                    TitanSpriteSheet::None => {
                        let rect_id = RectId {
                            image_index: titan_entry_index,
                            position: Position::ZERO,
                            size: image.size().into(),
                        };
                        rect_ids.push(rect_id);
                    }
                    TitanSpriteSheet::Homogeneous {
                        tile_size,
                        columns,
                        rows,
                        padding,
                        offset,
                    } => {
                        for i in 0..rows {
                            for j in 0..columns {
                                let padding = padding.unwrap_or(UVec2::ZERO);
                                let offset = offset.unwrap_or(UVec2::ZERO);
                                let x = j * tile_size.x as usize
                                    + offset.x as usize
                                    + ((1 + 2 * j) * padding.x as usize);
                                let y = i * tile_size.y as usize
                                    + offset.y as usize
                                    + ((1 + 2 * i) * padding.y as usize);

                                let rect_id = RectId {
                                    image_index: titan_entry_index,
                                    position: Position { x, y },
                                    size: tile_size.into(),
                                };
                                rect_ids.push(rect_id);
                            }
                        }
                    }
                    TitanSpriteSheet::Heterogeneous(rects) => {
                        rects.into_iter().for_each(|(position, size)| {
                            let rect_id = RectId {
                                image_index: titan_entry_index,
                                position,
                                size,
                            };
                            rect_ids.push(rect_id);
                        })
                    }
                }

                /* Save image to vec */
                images.push(image);
            }

            /* Query rect to place */
            let mut rects_to_place = GroupedRectsToPlace::<RectId>::new();
            rect_ids.iter().for_each(|rect_id| {
                let rect_to_insert =
                    RectToInsert::new(rect_id.size.width as u32, rect_id.size.height as u32, 1);
                rects_to_place.push_rect(rect_id.clone(), None, rect_to_insert);
            });

            /* Resolve the rect packing */
            let texture_atlas_size = UVec2::new(72, 72); /* TODO: Other size and multiple tries */
            let mut target_bins = BTreeMap::new();
            target_bins.insert(
                0,
                TargetBin::new(texture_atlas_size.x, texture_atlas_size.y, 1),
            );
            let rectangle_placements = pack_rects(
                &rects_to_place,
                &mut target_bins,
                &volume_heuristic,
                &contains_smallest_box,
            )
            .unwrap();

            /* Create new image from rects and source images */
            let pixel_format_size = 4; /* TODO: Proper */
            let texture_format = TextureFormat::Rgba8UnormSrgb; /* TODO: Proper */
            let mut texture_atlas_image = Image::new(
                Extent3d {
                    width: texture_atlas_size.x,
                    height: texture_atlas_size.y,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                vec![0; pixel_format_size * (texture_atlas_size.x * texture_atlas_size.y) as usize],
                texture_format,
            );
            let texture_atlas_textures: Vec<Rect> = rect_ids
                .into_iter()
                .map(|rect_id| {
                    let image = images.get(rect_id.image_index).unwrap();
                    let position = rect_id.position;

                    let (_, packed_location) = rectangle_placements
                        .packed_locations()
                        .get(&rect_id)
                        .unwrap();

                    /* Fill out the texture atlas */
                    copy_rect_image_to_texture_atlas(
                        &mut texture_atlas_image,
                        packed_location,
                        image,
                        position,
                    );

                    packed_location.as_rect()
                })
                .collect();

            // Create a Handle from the Image
            let texture_atlas_image_size = texture_atlas_size.as_vec2();
            let texture_atlas_image_handle =
                load_context.add_loaded_labeled_asset("image", texture_atlas_image.into());

            let mut texture_atlas =
                TextureAtlas::new_empty(texture_atlas_image_handle, texture_atlas_image_size);
            texture_atlas_textures.into_iter().for_each(|texture| {
                texture_atlas.add_texture(texture);
            });

            Ok(texture_atlas)
        })
    }

    fn extensions(&self) -> &[&str] {
        FILE_EXTENSIONS
    }
}

/* TODO: attributes like TextureAtlasBuilder */
#[derive(Debug, Deserialize)]
struct Titan {
    #[serde(default)]
    configuration: Option<()>,
    textures: Vec<TitanEntry>,
}

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
    Homogeneous {
        tile_size: UVec2,
        columns: usize,
        rows: usize,
        #[serde(default)]
        padding: Option<UVec2>,
        #[serde(default)]
        offset: Option<UVec2>,
    },
    Heterogeneous(Vec<(Position, Size)>),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Ord, PartialOrd)]
struct RectId {
    image_index: usize,
    position: Position,
    size: Size,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Ord, PartialOrd, Copy, Deserialize)]
struct Position {
    x: usize,
    y: usize,
}

impl Position {
    const ZERO: Self = Self { x: 0, y: 0 };
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Ord, PartialOrd, Copy, Deserialize)]
struct Size {
    width: usize,
    height: usize,
}

impl From<UVec2> for Size {
    fn from(value: UVec2) -> Self {
        Self {
            width: value.x as usize,
            height: value.y as usize,
        }
    }
}

impl From<&UVec2> for Size {
    fn from(value: &UVec2) -> Self {
        Self {
            width: value.x as usize,
            height: value.y as usize,
        }
    }
}

fn copy_rect_image_to_texture_atlas(
    texture_atlas: &mut Image,
    location: &PackedLocation,
    image: &Image,
    position: Position,
) {
    let format_size = texture_atlas.texture_descriptor.format.pixel_size();
    let rect_x = location.x() as usize;
    let rect_y = location.y() as usize;
    let rect_width = location.width() as usize;
    let rect_height = location.height() as usize;
    let texture_atlas_width = texture_atlas.width() as usize;

    /* Copy over from rect image, row by row */
    for i in 0..rect_height {
        let texture_atlas_begin = (rect_x + ((rect_y + i) * texture_atlas_width)) * format_size;
        let texture_atlas_end = texture_atlas_begin + rect_width * format_size;
        let data_begin = (position.x + (position.y + i) * image.width() as usize) * format_size;
        let data_end = data_begin + rect_width * format_size;

        texture_atlas.data[texture_atlas_begin..texture_atlas_end]
            .copy_from_slice(&image.data[data_begin..data_end]);
    }
}

trait AsRect {
    fn as_rect(&self) -> Rect;
}

impl AsRect for PackedLocation {
    fn as_rect(&self) -> Rect {
        Rect {
            min: Vec2::new(self.x() as f32, self.y() as f32),
            max: Vec2::new(
                (self.x() + self.width()) as f32,
                (self.y() + self.height()) as f32,
            ),
        }
    }
}
