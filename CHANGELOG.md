v0.6.0
================================================================================================================================
Update to bevy 0.13.
AssetLoader returns a `TextureAtlas` Handle that cotains handles for an Image and the TextureAtlasLayout. See examples for
changed usage.
Internally we now use the TextureAtlasBuilder.
Remove `always_pack` option. Default behaviour is now `true`.
Internally we now use serde features of glam and wgpu.
