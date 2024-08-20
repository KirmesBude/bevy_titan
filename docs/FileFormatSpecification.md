# Titan RON file format specification.

## Titan
| Field         | Type                   | Necessity | Description |
|---------------|------------------------|-----------|-------------|
| configuration | [TitanConfiguration]   | optional  | Configuration struct to control parameters of the packing algorithm and asset loader. |
| textures      | Vector of [TitanEntry] | mandatory | All textures of this texture atlas. Order is preserved when retrieving a specific sprite from the atlas by index. Can not be empty. |

## TitanConfiguration
| Field                  | Type                       | Necessity | Description |
|------------------------|----------------------------|-----------|-------------|
| initial_size           | [UVec2]                    | optional  | Starting size of the combined texture atlas for the packing process. Default value (256,256). |
| max_size               | [UVec2]                    | optional  | Maximum size that the combined texture atlas is allowed to grow to during the packing process. Default value (2048,2048). |
| format                 | String of [TextureFormat]  | optional  | Texture format of the combined texture atlas. Default value Rgba8UnormSrgb. |
| auto_format_conversion | bool                       | optional  | Automatically attempt to convert all textures into the texture format given for the combined texture atlas. Default value true. |
| padding                | [UVec2]                    | optional  | Padding between the sprites in the combined texture atlas. Default value (0,0). |

## TitanEntry
| Field        | Type               | Necessity | Description |
|--------------|--------------------|-----------|-------------|
| path         | String             | mandatory | Full file path to the underlying image asset. Relative to the assets folder. |
| sprite_sheet | [TitanSpriteSheet] | optional  | Enum to control how the image asset is interpreted for packing into a combined texture atlas. Default value None. |

## TitanSpriteSheet
| Variant       | Description |
|---------------|-------------|
| None          | Image asset is a single image. Default variant. |
| Homogeneous   | Image asset is a homogeneous sprite sheet. |
| Heterogeneous | Image asset is a heterogeneous sprite sheet. List of rects per sprite expressed by a tuple of [UVec2]. The first member is the top left starting position of the rectangle and the second member is the width and the height.|

## TitanSpriteSheet::Homogeneous
| Field     | Type     | Necessity | Description |
|-----------|----------|-----------|-------------|
| tile_size | [UVec2]  | mandatory | Size of each sprite in the sprite sheet. |
| columns   | u32      | mandatory | The amount of columns in the sprite sheet. |
| rows      | u32      | mandatory | The amount of rows in the sprite sheet. |
| padding   | [UVec2]  | optional  | Padding between the sprites in the sprite sheet. Default value (0,0). |
| offset    | [UVec2]  | optional  | Offset from (0,0) where the first sprite in the sprite sheet is located. Default value (0,0). |

[TitanConfiguration]: #titanconfiguration
[TitanEntry]: #titanentry
[UVec2]: https://docs.rs/bevy/latest/bevy/math/struct.UVec2.html
[TextureFormat]: https://docs.rs/bevy/latest/bevy/render/render_resource/enum.TextureFormat.html
[TitanSpriteSheet]: #titanspritesheet