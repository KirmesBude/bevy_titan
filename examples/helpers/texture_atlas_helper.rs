use bevy::prelude::*;

pub fn spawn_entire_texture_atlas(mut commands: Commands, texture: Handle<Image>) {
    commands.spawn(SpriteBundle {
        texture: texture,
        transform: Transform::from_translation(Vec3::new(-300.0, 0.0, -1.0))
            .with_scale(Vec3::splat(3.0)),
        ..Default::default()
    });
}
