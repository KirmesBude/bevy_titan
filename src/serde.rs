//! This module defines all types necessary for deserialization of titan ron files.
//!

use bevy::{math::UVec2, render::render_resource::TextureFormat};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct Titan {
    #[serde(default)]
    pub(crate) configuration: TitanConfiguration,
    pub(crate) textures: Vec<TitanEntry>,
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct TitanConfiguration {
    #[serde(default)]
    pub(crate) always_pack: bool, /* TODO: Support or remove */
    #[serde(default = "default_initial_size")]
    pub(crate) initial_size: UVec2,
    #[serde(default = "default_max_size")]
    pub(crate) max_size: UVec2,
    #[serde(default = "default_format")]
    pub(crate) format: TextureFormat,
    #[serde(default = "default_auto_format_conversion")]
    pub(crate) auto_format_conversion: bool,
    #[serde(default = "default_padding")]
    pub(crate) padding: UVec2,
}

impl Default for TitanConfiguration {
    fn default() -> Self {
        Self {
            always_pack: bool::default(),
            initial_size: default_initial_size(),
            max_size: default_max_size(),
            format: default_format(),
            auto_format_conversion: default_auto_format_conversion(),
            padding: default_padding(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct TitanEntry {
    pub(crate) path: String,
    #[serde(default)]
    pub(crate) sprite_sheet: TitanSpriteSheet,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub(crate) enum TitanSpriteSheet {
    #[default]
    None,
    Homogeneous {
        tile_size: UVec2,
        columns: u32,
        rows: u32,
        #[serde(default = "default_padding")]
        padding: UVec2,
        #[serde(default = "default_offset")]
        offset: UVec2,
    },
    Heterogeneous(Vec<(UVec2, UVec2)>),
}

#[inline]
const fn default_initial_size() -> UVec2 {
    UVec2::new(256, 265)
}

#[inline]
const fn default_max_size() -> UVec2 {
    UVec2::new(2048, 2048)
}

#[inline]
const fn default_format() -> TextureFormat {
    TextureFormat::Rgba8UnormSrgb
}

#[inline]
const fn default_auto_format_conversion() -> bool {
    true
}

#[inline]
const fn default_padding() -> UVec2 {
    UVec2::ZERO
}

#[inline]
const fn default_offset() -> UVec2 {
    UVec2::ZERO
}
