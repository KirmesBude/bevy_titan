//! Adapted from https://github.com/bevyengine/bevy/blob/v0.9.1/examples/2d/sprite_sheet.rs
//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

#[path = "helpers/animation_helper.rs"]
mod animation_helper;

use animation_helper::{animate_sprite, AnimationTimer};
use bevy::prelude::*;
use bevy_titan::SpriteSheetLoaderPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_plugins(SpriteSheetLoaderPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, animate_sprite)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_atlas_texture_handle = asset_server.load("gabe-idle-run.titan.ron#texture");
    let texture_atlas_layout_handle = asset_server.load("gabe-idle-run.titan.ron#layout");
    commands.spawn(Camera2d);
    commands.spawn((
        Sprite {
            image: texture_atlas_texture_handle.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout_handle,
                ..Default::default()
            }),
            ..Default::default()
        },
        Transform::from_scale(Vec3::splat(6.0)),
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}
