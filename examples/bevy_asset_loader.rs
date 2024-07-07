//! Adapted from https://github.com/NiklasEi/bevy_asset_loader/blob/b372b972e67a2e4b076442d23dcaff07083da611/bevy_asset_loader/examples/atlas_from_grid.rs
//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

#[path = "helpers/animation_helper.rs"]
mod animation_helper;

use animation_helper::{animate_sprite, AnimationTimer};
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_titan::SpriteSheetLoaderPlugin;

/// This example demonstrates how to load a texture atlas from a sprite sheet
///
/// Requires the feature '2d'
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .init_state::<MyStates>()
        .add_plugins(SpriteSheetLoaderPlugin)
        .add_loading_state(
            LoadingState::new(MyStates::AssetLoading)
                .continue_to_state(MyStates::Next)
                .load_collection::<MyAssets>(),
        )
        .add_systems(OnEnter(MyStates::Next), setup)
        .add_systems(Update, animate_sprite.run_if(in_state(MyStates::Next)))
        .run();
}

#[derive(AssetCollection, Resource)]
struct MyAssets {
    #[asset(path = "gabe-idle-run.titan#texture")]
    atlas_texture: Handle<Image>,
    #[asset(path = "gabe-idle-run.titan#layout")]
    texture_atlas_layout: Handle<TextureAtlasLayout>,
}

fn setup(mut commands: Commands, my_assets: Res<MyAssets>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteBundle {
            texture: my_assets.atlas_texture.clone(),
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..default()
        },
        TextureAtlas {
            layout: my_assets.texture_atlas_layout.clone(),
            ..Default::default()
        },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum MyStates {
    #[default]
    AssetLoading,
    Next,
}
