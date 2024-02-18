use bevy::prelude::*;

#[derive(Component, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

pub fn animate_sprite(
    time: Res<Time>,
    texture_atlas_layouts: Res<Assets<TextureAtlasLayout>>,
    mut query: Query<(&mut AnimationTimer, &mut TextureAtlas)>,
) {
    for (mut timer, mut texture_atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            let texture_atlas_layout = texture_atlas_layouts.get(&texture_atlas.layout).unwrap();
            texture_atlas.index = (texture_atlas.index + 1) % texture_atlas_layout.textures.len();
        }
    }
}
