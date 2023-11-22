//! This module handles loading a TextureAtlas a titan ron file.
//!

use std::{collections::BTreeMap, path::Path};

use bevy::{
    asset::{io::Reader, AssetLoader, AssetPath, AsyncReadExt, LoadContext, LoadDirectError},
    math::{Rect, UVec2, Vec2},
    render::{
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::{Image, TextureFormatPixelInfo},
    },
    sprite::TextureAtlas,
    utils::BoxedFuture,
};
use rectangle_pack::{
    contains_smallest_box, pack_rects, volume_heuristic, GroupedRectsToPlace, PackedLocation,
    RectToInsert, RectanglePackError, RectanglePackOk, TargetBin,
};
use thiserror::Error;

use crate::serde::{Titan, TitanConfiguration, TitanEntry, TitanSpriteSheet, TitanUVec2};

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
    /// A FormatConversionError
    #[error("TextureFormat conversion failed for {0}: {1:?} to {2:?}")]
    FormatConversionError(String, TextureFormat, TextureFormat),
    /// A IncompatibleFormatError
    #[error("Placing texture {0} of format {1:?} into texture atlas of format {2:?}")]
    IncompatibleFormatError(String, TextureFormat, TextureFormat),
    /// A RectanglePackError
    #[error("Could not pack all rectangles for the given size: {0}")]
    RectanglePackError(RectanglePackError),
    /// An NoEntriesError
    #[error("No entries were found")]
    NoEntriesError,
    /// An InvalidRectError
    #[error("InvalidRectError: {0}")]
    InvalidRectError(#[from] InvalidRectError),
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
            let titan = ron::de::from_bytes::<Titan>(&bytes)?;
            let configuration = titan.configuration;
            let titan_entries = titan.textures;

            if titan_entries.is_empty() {
                return Err(SpriteSheetLoaderError::NoEntriesError);
            }

            let rect_ids_len = titan_entries.iter().fold(0, |acc, titan_entry| {
                acc + match &titan_entry.sprite_sheet {
                    TitanSpriteSheet::None => 1,
                    TitanSpriteSheet::Homogeneous { columns, rows, .. } => {
                        (columns * rows) as usize
                    }
                    TitanSpriteSheet::Heterogeneous(vec) => vec.len(),
                }
            });
            let mut rect_ids = Vec::with_capacity(rect_ids_len);
            let images_len = titan_entries.len();
            let mut images = Vec::with_capacity(images_len);
            for (titan_entry_index, titan_entry) in titan_entries.into_iter().enumerate() {
                /* Load the image */
                let titan_entry_path = titan_entry.path.clone();
                let image_asset_path = AssetPath::from_path(Path::new(&titan_entry_path));
                let image = load_context.load_direct(image_asset_path).await?;
                let image = image
                    .take::<Image>()
                    .ok_or(SpriteSheetLoaderError::NotAnImage(titan_entry_path.clone()))?;

                /* Get and insert all rects */
                push_rect_ids(&mut rect_ids, titan_entry, titan_entry_index, image.size())?;

                /* Save image to vec */
                let image = if configuration.auto_format_conversion {
                    image.convert(*configuration.format).ok_or(
                        SpriteSheetLoaderError::FormatConversionError(
                            titan_entry_path,
                            image.texture_descriptor.format,
                            *configuration.format,
                        ),
                    )?
                } else {
                    if image.texture_descriptor.format != *configuration.format {
                        return Err(SpriteSheetLoaderError::IncompatibleFormatError(
                            titan_entry_path,
                            image.texture_descriptor.format,
                            *configuration.format,
                        ));
                    }
                    image
                };
                images.push(image);
            }

            let (texture_atlas_size, texture_atlas_image, texture_atlas_textures) =
                place_rects_and_create_texture_atlas_image(images, rect_ids, configuration)?;

            // Create a Handle from the Image
            let texture_atlas_image_size = texture_atlas_size.into();
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

impl From<RectanglePackError> for SpriteSheetLoaderError {
    fn from(value: RectanglePackError) -> Self {
        Self::RectanglePackError(value)
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Ord, PartialOrd, Copy)]
struct RectId {
    image_index: usize,
    position: TitanUVec2,
    size: TitanUVec2,
}

impl RectId {
    fn new_with_validation(
        image_index: usize,
        position: TitanUVec2,
        size: TitanUVec2,
        image_size: UVec2,
    ) -> Option<Self> {
        let bound: UVec2 = (position + size).into();

        if (bound.x > image_size.x) || (bound.y > image_size.y) {
            None
        } else {
            Some(Self {
                image_index,
                position,
                size,
            })
        }
    }
}

fn copy_rect_image_to_texture_atlas(
    texture_atlas: &mut Image,
    location: &PackedLocation,
    image: &Image,
    position: TitanUVec2,
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
        let data_begin = (position.x() as usize
            + (position.y() as usize + i) * image.width() as usize)
            * format_size;
        let data_end = data_begin + rect_width * format_size;

        texture_atlas.data[texture_atlas_begin..texture_atlas_end]
            .copy_from_slice(&image.data[data_begin..data_end]);
    }
}

fn place_rects_and_create_texture_atlas_image(
    mut images: Vec<Image>,
    rect_ids: Vec<RectId>,
    configuration: TitanConfiguration,
) -> Result<(TitanUVec2, Image, Vec<Rect>), RectanglePackError> {
    if images.len() > 1 {
        /* Query rect to place */
        let mut rects_to_place = GroupedRectsToPlace::<RectId>::new();
        rect_ids.iter().for_each(|rect_id| {
            let rect_to_insert = RectToInsert::new(rect_id.size.width(), rect_id.size.height(), 1);
            rects_to_place.push_rect(*rect_id, None, rect_to_insert);
        });

        /* Resolve the rect packing */
        let mut texture_atlas_size = TitanUVec2(
            configuration.initial_size.width(),
            configuration.initial_size.height(),
        );
        let rectangle_placements = loop {
            let mut target_bins = BTreeMap::new();
            target_bins.insert(
                0,
                TargetBin::new(texture_atlas_size.x(), texture_atlas_size.y(), 1),
            );
            match pack_rects(
                &rects_to_place,
                &mut target_bins,
                &volume_heuristic,
                &contains_smallest_box,
            ) {
                Ok(rectangle_placements) => break rectangle_placements,
                Err(err) => {
                    if texture_atlas_size >= configuration.max_size {
                        return Err(err);
                    }
                    texture_atlas_size = TitanUVec2(
                        (texture_atlas_size.width() * 2).min(configuration.max_size.width()),
                        (texture_atlas_size.height() * 2).min(configuration.max_size.height()),
                    );
                }
            }
        };

        /* Create new image from rects and source images */
        let (texture_atlas_image, texture_atlas_textures) = create_texture_atlas_image(
            configuration,
            texture_atlas_size,
            rect_ids,
            rectangle_placements,
            images,
        );

        Ok((
            texture_atlas_size,
            texture_atlas_image,
            texture_atlas_textures,
        ))
    } else {
        Ok((
            images[0].size().into(),
            images.remove(0),
            rect_ids.iter().map(|rect_id| rect_id.as_rect()).collect(),
        ))
    }
}

fn create_texture_atlas_image(
    configuration: TitanConfiguration,
    texture_atlas_size: TitanUVec2,
    rect_ids: Vec<RectId>,
    rectangle_placements: RectanglePackOk<RectId, u32>,
    images: Vec<Image>,
) -> (Image, Vec<Rect>) {
    let texture_format = *configuration.format;
    let mut texture_atlas_image = Image::new(
        Extent3d {
            width: texture_atlas_size.width(),
            height: texture_atlas_size.height(),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        vec![
            0;
            configuration.format.pixel_size()
                * (texture_atlas_size.width() * texture_atlas_size.height()) as usize
        ],
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

    (texture_atlas_image, texture_atlas_textures)
}

/// InvalidRectError
#[derive(Debug, Error)]
#[error("Rect with position {0} and size {1} is invalid for image {2}")]
pub struct InvalidRectError(UVec2, UVec2, String);

fn push_rect_ids(
    rect_ids: &mut Vec<RectId>,
    titan_entry: TitanEntry,
    titan_entry_index: usize,
    image_size: UVec2,
) -> Result<(), InvalidRectError> {
    match titan_entry.sprite_sheet {
        TitanSpriteSheet::None => {
            let rect_id = RectId {
                image_index: titan_entry_index,
                position: TitanUVec2::ZERO,
                size: image_size.into(),
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
                    /* TODO: Simplify with Add/Multiply implementations on TitanUVec2 */
                    let x = j * tile_size.width() + offset.x() + ((1 + 2 * j) * padding.x());
                    let y = i * tile_size.height() + offset.y() + ((1 + 2 * i) * padding.y());
                    let position = TitanUVec2(x, y);

                    let rect_id = RectId::new_with_validation(
                        titan_entry_index,
                        position,
                        tile_size,
                        image_size,
                    )
                    .ok_or(InvalidRectError(
                        position.into(),
                        tile_size.into(),
                        titan_entry.path.clone(),
                    ))?;

                    rect_ids.push(rect_id);
                }
            }
        }
        TitanSpriteSheet::Heterogeneous(rects) => {
            for (position, size) in rects {
                let rect_id =
                    RectId::new_with_validation(titan_entry_index, position, size, image_size)
                        .ok_or(InvalidRectError(
                            position.into(),
                            size.into(),
                            titan_entry.path.clone(),
                        ))?;
                rect_ids.push(rect_id);
            }
        }
    }
    Ok(())
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

impl AsRect for RectId {
    fn as_rect(&self) -> Rect {
        Rect {
            min: self.position.into(),
            max: (self.position + self.size).into(),
        }
    }
}
