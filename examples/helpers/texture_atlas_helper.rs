use bevy::prelude::*;

pub fn spawn_entire_texture_atlas(
    mut commands: Commands,
    mut asset_event_evr: EventReader<AssetEvent<TextureAtlas>>,
    texture_atlas_assets: Res<Assets<TextureAtlas>>,
) {
    for ev in asset_event_evr.read() {
        if let AssetEvent::Added { id } = ev {
            let texture_atlas = texture_atlas_assets.get(*id).unwrap();

            commands.spawn(SpriteBundle {
                texture: texture_atlas.texture.clone(),
                transform: Transform::from_translation(Vec3::new(-300.0, 0.0, -1.0))
                    .with_scale(Vec3::splat(3.0)),
                ..Default::default()
            });
        }
    }
}
