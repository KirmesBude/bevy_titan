//! This crate allows you to directly load a TextureAtlas from a titan ron file.
//!
//! `bevy_titan` introduces a definition of a titan ron file and the corresponding [`SpriteSheetLoader`](crate::asset_loader::SpriteSheetLoader).
//! Assets with the 'titan' extension can be loaded just like any other asset via the [`AssetServer`](::bevy::asset::AssetServer)
//! and will yield a [`TextureAtlas`](::bevy::sprite::TextureAtlas) [`Handle`](::bevy::asset::Handle).
//!
//! ### `spritesheet.titan`
//! ```rust,ignore
//! Titan ( /* The explicit type name can be omitted */
//!     configuration: ( /* This is optional */
//!         always_pack: true, /* This is optional; false by default; If false, this will skip the texture packing step in case only a single texture is provided */
//!         initial_size: (128, 128), /* This is optional; (256, 256) by default; Initial size for the packing algorithm */
//!         max_size: (1024, 1024) , /* This is optional; (2048, 2048) by default; Max size for the packing algorithm */
//!         format: "Rgba8UnormSrgb", /* This is optional; Rgba8UnormSrgb; TexureFormat (see bevy::render::render_resource::TextureFormat) of the resulting TextureAtlas Image */
//!         auto_format_conversion: false, /* This is optional; true by default; Automatically converts all textures to the format provided */
//!         padding: (2, 2), /* This is optional; (0, 0) by default; Padding between textures in the resulting TextureAtlas Image */
//!     ),
//!     textures: [ /* This is mandatory and needs to contain at least one entry */
//!         (
//!             path: "homogeneous_sprite_sheet.png", /* Path to an image from AssetFolder */
//!             sprite_sheet: Homogeneous (
//!                 tile_size: (24, 24),
//!                 columns: 7,
//!                 rows: 1,
//!                 offset: (10, 10), /* This is optional; (0, 0) by default; Offset from (0, 0) in the image that the first texture starts */
//!                 padding: (2, 2), /* This is optional; (0, 0) by default; Padding on each side of the texture */
//!             )
//!         ),
//!         (
//!             path: "heterogeneous_sprite_sheet.png", /* Path to an image from AssetFolder */
//!             sprite_sheet: Heterogeneous (
//!                 [
//!                     (
//!                         (0, 0), /* Position that this texture inside the image starts (from top left) */
//!                         (24, 24) /* Size of the texture */
//!                     ),
//!                     (
//!                         (24, 0),
//!                         (24, 24)
//!                     ),
//!                     /* You can continue with as many textures as you want */
//!                 ]
//!             )
//!         ),
//!         (
//!             path: "sprite.png", /* Path to an image from AssetFolder */
//!             sprite_sheet: None, /* Can be omitted for single images */
//!         )
//!     ]
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
    asset::AssetApp,
    prelude::{App, Plugin},
};

pub mod asset_loader;
pub mod serde;

/// Adds support for spritesheet manifest files loading to the app.
pub struct SpriteSheetLoaderPlugin;

impl Plugin for SpriteSheetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset_loader::<asset_loader::SpriteSheetLoader>();
    }
}

/// `use bevy_titan::prelude::*;` to import common components and plugins.
pub mod prelude {
    pub use crate::SpriteSheetLoaderPlugin;
}
