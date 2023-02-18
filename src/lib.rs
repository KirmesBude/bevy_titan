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
const FILE_EXTENSIONS: &[&str] = &["titan"];

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
#[derive(Debug, Deserialize)]
struct SpriteSheetManifest {
    path: String,
    tile_size: Rect,
    columns: usize,
    rows: usize,
    #[serde(default)]
    padding: Option<Rect>,
    #[serde(default)]
    offset: Option<Rect>,
}

/// Helper struct to represent Vec2.
#[derive(Debug, Deserialize)]
struct Rect {
    w: f32,
    h: f32,
}

impl From<Rect> for Vec2 {
    fn from(value: Rect) -> Self {
        Self::new(value.w, value.h)
    }
}
