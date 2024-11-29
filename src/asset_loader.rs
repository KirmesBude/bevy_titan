//! This module handles loading a TextureAtlas from a titan ron file.
//!
//! `bevy_titan` introduces a definition of a titan ron file and the corresponding [`SpriteSheetLoader`].
//! Assets with the 'titan' extension can be loaded just like any other asset via the [`AssetServer`](::bevy::asset::AssetServer)
//! and will yield a [`TextureAtlas`] [`Handle`](::bevy::asset::Handle).

use std::path::Path;

use bevy::{
    asset::{io::Reader, Asset, AssetLoader, AssetPath, Handle, LoadContext, LoadDirectError},
    image::TextureFormatPixelInfo,
    math::{URect, UVec2},
    prelude::Image,
    reflect::Reflect,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension},
    },
    sprite::{TextureAtlasBuilder, TextureAtlasBuilderError, TextureAtlasLayout},
};
use thiserror::Error;

use crate::serde::{Titan, TitanEntry, TitanSpriteSheet};

/// Loader for spritesheet manifest files written in ron. Loads a TextureAtlas asset.
#[derive(Default)]
pub struct SpriteSheetLoader;

/// Possible errors that can be produced by [`SpriteSheetLoader`].
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SpriteSheetLoaderError {
    /// An [IOError](std::io::Error).
    #[error("Could not load file: {0}")]
    IoError(#[from] std::io::Error),
    /// A [RonSpannedError](ron::error::SpannedError).
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
    /// A [`LoadDirectError``].
    #[error("Could not load: {0}")]
    LoadDirectError(#[from] LoadDirectError),
    /// A NotAnImageError.
    #[error("Loading from {0} does not provide Image")]
    NotAnImageError(String),
    /// A [`TextureAtlasBuilderError`].
    #[error("TextureAtlasBuilderError: {0}")]
    TextureAtlasBuilderError(#[from] TextureAtlasBuilderError),
    /// A NoEntriesError
    #[error("No entries were found")]
    NoEntriesError,
    /// An [`InvalidRectError`].
    #[error("InvalidRectError: {0}")]
    InvalidRectError(#[from] InvalidRectError),
    /// A SizeMismatchError.
    #[error("Configured initial size {0} is bigger than max size {1}")]
    SizeMismatchError(UVec2, UVec2),
}

/// InvalidRectError.
#[derive(Debug, Error)]
#[error("Rect with min {0} and max {1} is invalid for image {2}")]
pub struct InvalidRectError(UVec2, UVec2, String);

/// File extension for spritesheet manifest files written in ron.
pub const FILE_EXTENSIONS: &[&str] = &["titan.ron", "titan"];

/// TextureAtlas Asset
#[derive(Debug, Asset, Reflect)]
pub struct TextureAtlas {
    /// Atlas Texture Image
    pub texture: Handle<Image>,
    /// Texture Atlas Layout
    pub layout: Handle<TextureAtlasLayout>,
}

impl AssetLoader for SpriteSheetLoader {
    type Asset = TextureAtlas;
    type Settings = ();
    type Error = SpriteSheetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let titan = ron::de::from_bytes::<Titan>(&bytes)?;

        let configuration = titan.configuration;
        if configuration.max_size.x < configuration.initial_size.x
            || configuration.max_size.y < configuration.initial_size.y
        {
            return Err(SpriteSheetLoaderError::SizeMismatchError(
                configuration.initial_size,
                configuration.max_size,
            ));
        }

        let titan_entries = titan.textures;
        if titan_entries.is_empty() {
            return Err(SpriteSheetLoaderError::NoEntriesError);
        }

        let images_len = titan_entries.iter().fold(0, |acc, titan_entry| {
            acc + match &titan_entry.sprite_sheet {
                TitanSpriteSheet::None => 1,
                TitanSpriteSheet::Homogeneous { columns, rows, .. } => (columns * rows) as usize,
                TitanSpriteSheet::Heterogeneous(vec) => vec.len(),
            }
        });
        let mut images = Vec::with_capacity(images_len);
        for titan_entry in titan_entries.into_iter() {
            /* Load the image */
            let titan_entry_path = titan_entry.path.clone();
            let image_asset_path = AssetPath::from_path(Path::new(&titan_entry_path));
            let image = load_context
                .loader()
                .immediate()
                .load(image_asset_path)
                .await?;

            /* Get and insert all rects */
            push_textures(&mut images, titan_entry, image.take())?;
        }

        let mut texture_atlas_builder = TextureAtlasBuilder::default();
        texture_atlas_builder
            .initial_size(configuration.initial_size)
            .max_size(configuration.max_size)
            .format(configuration.format)
            .auto_format_conversion(configuration.auto_format_conversion)
            .padding(configuration.padding);
        for image in &images {
            texture_atlas_builder.add_texture(None, image);
        }
        let (texture_atlas_layout, _, atlas_texture) = texture_atlas_builder.build()?;

        let atlas_texture_handle =
            load_context.add_loaded_labeled_asset("texture", atlas_texture.into());
        let texture_atlas_layout_handle =
            load_context.add_loaded_labeled_asset("layout", texture_atlas_layout.into());

        let texture_atlas = TextureAtlas {
            texture: atlas_texture_handle,
            layout: texture_atlas_layout_handle,
        };

        Ok(texture_atlas)
    }

    fn extensions(&self) -> &[&str] {
        FILE_EXTENSIONS
    }
}

fn push_textures(
    images: &mut Vec<Image>,
    titan_entry: TitanEntry,
    texture: Image,
) -> Result<(), InvalidRectError> {
    match titan_entry.sprite_sheet {
        TitanSpriteSheet::None => {
            images.push(texture);
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
                    let min = UVec2::new(j, i) * tile_size
                        + offset
                        + (UVec2::new(1 + 2 * j, 1 + 2 * i) * padding);
                    let max = min + tile_size;
                    let rect = URect::from_corners(min, max);

                    let image = extract_texture_from_rect(&texture, rect)?;

                    images.push(image);
                }
            }
        }
        TitanSpriteSheet::Heterogeneous(rects) => {
            for (position, size) in rects {
                let min = position;
                let max = min + size;
                let rect = URect::from_corners(min, max);

                let image = extract_texture_from_rect(&texture, rect)?;

                images.push(image);
            }
        }
    }

    Ok(())
}

fn extract_texture_from_rect(image: &Image, rect: URect) -> Result<Image, InvalidRectError> {
    if (rect.max.x > image.size().x) || (rect.max.y > image.size().y) {
        Err(InvalidRectError(rect.min, rect.max, String::from("Test")))
    } else {
        let format_size = image.texture_descriptor.format.pixel_size();
        let rect_size = UVec2::new(rect.max.x - rect.min.x, rect.max.y - rect.min.y);
        let mut data: Vec<u8> = vec![0; (rect_size.x * rect_size.y) as usize * format_size];

        for i in 0..rect_size.y {
            let data_begin = (rect_size.x * i) as usize * format_size;
            let data_end = data_begin + rect_size.x as usize * format_size;
            let texture_atlas_rect_begin = (rect.min.x as usize
                + (rect.min.y + i) as usize * image.width() as usize)
                * format_size;
            let texture_atlas_rect_end =
                texture_atlas_rect_begin + rect_size.x as usize * format_size;

            data[data_begin..data_end]
                .copy_from_slice(&image.data[texture_atlas_rect_begin..texture_atlas_rect_end]);
        }

        let image = Image::new(
            Extent3d {
                width: rect_size.x,
                height: rect_size.y,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            image.texture_descriptor.format,
            RenderAssetUsages::MAIN_WORLD,
        );
        Ok(image)
    }
}

#[cfg(test)]
mod tests {
    /* TODO: Tests */
}
