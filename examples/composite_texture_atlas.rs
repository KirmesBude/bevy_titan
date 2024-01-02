//! Adapted from https://github.com/bevyengine/bevy/blob/v0.9.1/examples/2d/sprite_sheet.rs
//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.

use bevy::prelude::*;
use bevy_titan::SpriteSheetLoaderPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_plugins(SpriteSheetLoaderPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (animate_sprite, spawn_entire_texture_atlas))
        .run();
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = (sprite.index + 1) % texture_atlas.textures.len();
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let texture_atlas_handle = asset_server.load("composite-texture-atlas.titan");
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..default()
        },
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}

fn spawn_entire_texture_atlas(
    mut commands: Commands,
    mut asset_event_evr: EventReader<AssetEvent<TextureAtlas>>,
    texture_atlas_assets: Res<Assets<TextureAtlas>>,
) {
    for ev in asset_event_evr.read() {
        match ev {
            AssetEvent::Added { id } => {
                let texture_atlas = texture_atlas_assets.get(*id).unwrap();

                commands.spawn(SpriteBundle {
                    texture: texture_atlas.texture.clone(),
                    transform: Transform::from_translation(Vec3::new(-320.0, 0.0, -1.0))
                        .with_scale(Vec3::splat(3.0)),
                    ..Default::default()
                });
            }
            _ => {}
        }
    }
}
