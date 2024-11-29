use bevy::prelude::*;

pub fn spawn_entire_texture_atlas(mut commands: Commands, image: Handle<Image>) {
    commands.spawn((
        Sprite {
            image: image,
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(-300.0, 0.0, -1.0)).with_scale(Vec3::splat(3.0)),
    ));
}
