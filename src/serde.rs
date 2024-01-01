//! This module defines all types necessary for deserialization of titan ron files.
//!

use std::ops::{Add, Mul};

use bevy::{
    math::{UVec2, Vec2},
    prelude::Deref,
    render::render_resource::TextureFormat,
};
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
    pub(crate) always_pack: bool,
    #[serde(default = "default_initial_size")]
    pub(crate) initial_size: TitanUVec2,
    #[serde(default = "default_max_size")]
    pub(crate) max_size: TitanUVec2,
    #[serde(default = "default_format")]
    pub(crate) format: TitanTextureFormat,
    #[serde(default = "default_auto_format_conversion")]
    pub(crate) auto_format_conversion: bool,
    #[serde(default = "default_padding")]
    pub(crate) padding: TitanUVec2,
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

#[derive(Debug, Deref, Clone)]
pub(crate) struct TitanTextureFormat(TextureFormat);

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
        tile_size: TitanUVec2,
        columns: u32,
        rows: u32,
        #[serde(default = "default_padding")]
        padding: TitanUVec2,
        #[serde(default = "default_offset")]
        offset: TitanUVec2,
    },
    Heterogeneous(Vec<(TitanUVec2, TitanUVec2)>), /* TODO: This does not make is clear what is what. */
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Ord, PartialOrd, Copy, Deserialize)]
pub(crate) struct TitanUVec2(pub(crate) u32, pub(crate) u32);

impl TitanUVec2 {
    pub(crate) const ZERO: Self = Self(0, 0);

    pub(crate) fn x(&self) -> u32 {
        self.0
    }

    pub(crate) fn y(&self) -> u32 {
        self.1
    }

    pub(crate) fn width(&self) -> u32 {
        self.0
    }

    pub(crate) fn height(&self) -> u32 {
        self.1
    }
}

impl From<UVec2> for TitanUVec2 {
    fn from(value: UVec2) -> Self {
        Self(value.x, value.y)
    }
}

impl From<TitanUVec2> for UVec2 {
    fn from(value: TitanUVec2) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}

impl From<TitanUVec2> for Vec2 {
    fn from(value: TitanUVec2) -> Self {
        Self {
            x: value.0 as f32,
            y: value.1 as f32,
        }
    }
}

impl Add<TitanUVec2> for TitanUVec2 {
    type Output = Self;

    fn add(self, rhs: TitanUVec2) -> Self::Output {
        Self(self.0.add(rhs.0), self.1.add(rhs.1))
    }
}

impl Mul<u32> for TitanUVec2 {
    type Output = Self;

    fn mul(self, rhs: u32) -> Self::Output {
        Self(self.0.mul(rhs), self.1.mul(rhs))
    }
}

impl Mul<TitanUVec2> for u32 {
    type Output = TitanUVec2;

    fn mul(self, rhs: TitanUVec2) -> Self::Output {
        TitanUVec2(self.mul(rhs.0), self.mul(rhs.1))
    }
}

#[inline]
const fn default_initial_size() -> TitanUVec2 {
    TitanUVec2(256, 265)
}

#[inline]
const fn default_max_size() -> TitanUVec2 {
    TitanUVec2(2048, 2048)
}

#[inline]
const fn default_format() -> TitanTextureFormat {
    TitanTextureFormat(TextureFormat::Rgba8UnormSrgb)
}

#[inline]
const fn default_auto_format_conversion() -> bool {
    true
}

#[inline]
const fn default_padding() -> TitanUVec2 {
    TitanUVec2::ZERO
}

#[inline]
const fn default_offset() -> TitanUVec2 {
    TitanUVec2::ZERO
}

impl<'de> Deserialize<'de> for TitanTextureFormat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let texture_format = match s.as_str() {
            "R8Unorm" => TextureFormat::R8Unorm,
            "R8Snorm" => TextureFormat::R8Snorm,
            "R8Uint" => TextureFormat::R8Uint,
            "R8Sint" => TextureFormat::R8Sint,
            "R16Uint" => TextureFormat::R16Uint,
            "R16Sint" => TextureFormat::R16Sint,
            "R16Unorm" => TextureFormat::R16Unorm,
            "R16Snorm" => TextureFormat::R16Snorm,
            "R16Float" => TextureFormat::R16Float,
            "Rg8Unorm" => TextureFormat::Rg8Unorm,
            "Rg8Snorm" => TextureFormat::Rg8Snorm,
            "Rg8Uint" => TextureFormat::Rg8Uint,
            "Rg8Sint" => TextureFormat::Rg8Sint,
            "R32Uint" => TextureFormat::R32Uint,
            "R32Sint" => TextureFormat::R32Sint,
            "R32Float" => TextureFormat::R32Float,
            "Rg16Uint" => TextureFormat::Rg16Uint,
            "Rg16Sint" => TextureFormat::Rg16Sint,
            "Rg16Unorm" => TextureFormat::Rg16Unorm,
            "Rg16Snorm" => TextureFormat::Rg16Snorm,
            "Rg16Float" => TextureFormat::Rg16Float,
            "Rgba8Unorm" => TextureFormat::Rgba8Unorm,
            "Rgba8UnormSrgb" => TextureFormat::Rgba8UnormSrgb,
            "Rgba8Snorm" => TextureFormat::Rgba8Snorm,
            "Rgba8Uint" => TextureFormat::Rgba8Uint,
            "Rgba8Sint" => TextureFormat::Rgba8Sint,
            "Bgra8Unorm" => TextureFormat::Bgra8Unorm,
            "Bgra8UnormSrgb" => TextureFormat::Bgra8UnormSrgb,
            "Rgb9e5Ufloat" => TextureFormat::Rgb9e5Ufloat,
            "Rgb10a2Unorm" => TextureFormat::Rgb10a2Unorm,
            "Rg11b10Float" => TextureFormat::Rg11b10Float,
            "Rg32Uint" => TextureFormat::Rg32Uint,
            "Rg32Sint" => TextureFormat::Rg32Sint,
            "Rg32Float" => TextureFormat::Rg32Float,
            "Rgba16Uint" => TextureFormat::Rgba16Uint,
            "Rgba16Sint" => TextureFormat::Rgba16Sint,
            "Rgba16Unorm" => TextureFormat::Rgba16Unorm,
            "Rgba16Snorm" => TextureFormat::Rgba16Snorm,
            "Rgba16Float" => TextureFormat::Rgba16Float,
            "Rgba32Uint" => TextureFormat::Rgba32Uint,
            "Rgba32Sint" => TextureFormat::Rgba32Sint,
            "Rgba32Float" => TextureFormat::Rgba32Float,
            "Stencil8" => TextureFormat::Stencil8,
            "Depth16Unorm" => TextureFormat::Depth16Unorm,
            "Depth24Plus" => TextureFormat::Depth24Plus,
            "Depth24PlusStencil8" => TextureFormat::Depth24PlusStencil8,
            "Depth32Float" => TextureFormat::Depth32Float,
            "Depth32FloatStencil8" => TextureFormat::Depth32FloatStencil8,
            "Bc1RgbaUnorm" => TextureFormat::Bc1RgbaUnorm,
            "Bc1RgbaUnormSrgb" => TextureFormat::Bc1RgbaUnormSrgb,
            "Bc2RgbaUnorm" => TextureFormat::Bc2RgbaUnorm,
            "Bc2RgbaUnormSrgb" => TextureFormat::Bc2RgbaUnormSrgb,
            "Bc3RgbaUnorm" => TextureFormat::Bc3RgbaUnorm,
            "Bc3RgbaUnormSrgb" => TextureFormat::Bc3RgbaUnormSrgb,
            "Bc4RUnorm" => TextureFormat::Bc4RUnorm,
            "Bc4RSnorm" => TextureFormat::Bc4RSnorm,
            "Bc5RgUnorm" => TextureFormat::Bc5RgUnorm,
            "Bc5RgSnorm" => TextureFormat::Bc5RgSnorm,
            "Bc6hRgbUfloat" => TextureFormat::Bc6hRgbUfloat,
            "Bc6hRgbFloat" => TextureFormat::Bc6hRgbFloat,
            "Bc7RgbaUnorm" => TextureFormat::Bc7RgbaUnorm,
            "Bc7RgbaUnormSrgb" => TextureFormat::Bc7RgbaUnormSrgb,
            "Etc2Rgb8Unorm" => TextureFormat::Etc2Rgb8Unorm,
            "Etc2Rgb8UnormSrgb" => TextureFormat::Etc2Rgb8UnormSrgb,
            "Etc2Rgb8A1Unorm" => TextureFormat::Etc2Rgb8A1Unorm,
            "Etc2Rgb8A1UnormSrgb" => TextureFormat::Etc2Rgb8A1UnormSrgb,
            "Etc2Rgba8Unorm" => TextureFormat::Etc2Rgba8Unorm,
            "Etc2Rgba8UnormSrgb" => TextureFormat::Etc2Rgba8UnormSrgb,
            "EacR11Unorm" => TextureFormat::EacR11Unorm,
            "EacR11Snorm" => TextureFormat::EacR11Snorm,
            "EacRg11Unorm" => TextureFormat::EacRg11Unorm,
            "EacRg11Snorm" => TextureFormat::EacRg11Snorm,
            other => {
                return Err(serde::de::Error::custom(format!(
                    "Invalid variant '{}'",
                    other
                )));
            }
        };
        Ok(TitanTextureFormat(texture_format))
    }
}
