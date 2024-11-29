use bevy::prelude::*;

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

pub fn animate_sprite(
    time: Res<Time>,
    texture_atlas_layouts: Res<Assets<TextureAtlasLayout>>,
    mut query: Query<(&mut AnimationTimer, &mut Sprite)>,
) {
    for (mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            if let Some(ref mut texture_atlas) = sprite.texture_atlas.as_mut() {
                let texture_atlas_layout =
                    texture_atlas_layouts.get(&texture_atlas.layout).unwrap();
                texture_atlas.index =
                    (texture_atlas.index + 1) % texture_atlas_layout.textures.len();
            }
        }
    }
}
