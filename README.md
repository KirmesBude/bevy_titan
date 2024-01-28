# bevy_titan

[![crates.io](https://img.shields.io/crates/v/bevy_titan)](https://crates.io/crates/bevy_titan)
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-released%20version-lightblue)](https://github.com/bevyengine/bevy/blob/main/docs/plugins_guidelines.md#main-branch-tracking)
[![docs.rs](https://docs.rs/bevy_titan/badge.svg)](https://docs.rs/bevy_titan)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/bevyengine/bevy#license)

| bevy | bevy_titan   |
|------|--------------|
| 0.12 | 0.4.0, 0.5.0 |
| 0.11 | 0.3.0        |
| 0.10 | 0.2.0        |
| 0.9  | 0.1.1        |

## What is bevy_titan?

`bevy_titan` is a simple bevy plugin to load textures atlases from spritesheet manifest files written in ron.
It also supports creating a texture atlas from multiple sprites and even multiple sprite sheets.

## Quickstart


```toml, ignore
# In your Cargo.toml
bevy_titan = "0.5"
```

### homogeneous-sprite-sheet.titan
```rust, ignore
//! A basic example of a titan file for a homogeneous sprite sheet.
(
    textures: [
        (
            path: "path-to-homogeneous-sprite-sheet",
            sprite_sheet: Homogeneous (
                tile_size: (
                    32,
                    32,
                ),
                columns: 4,
                rows: 1,
            ),
        ),
    ]
)
```

### heterogeneous-sprite-sheet.titan
```rust, ignore
//! A basic example of a titan file for a heterogeneous sprite sheet.
(
    textures: [
        (
            path: "path-to-heterogeneous-sprite-sheet",
            sprite_sheet: Heterogeneous (
                [
                    (
                        (0, 0),
                        (16, 16),
                    ),
                    (
                        (16, 0),
                        (32, 128),
                    ),
                    (
                        (48, 0),
                        (64, 16),
                    ),
                ]
            ),
        ),
    ]
)
```

### main.rs
```rust, ignore
//! A basic example of how to create a TextureAtlas asset from a titan file.
use bevy::prelude::*;
use bevy_titan::SpriteSheetLoaderPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SpriteSheetLoaderPlugin)
        .add_systems(Startup, (setup, load_texture_atlas).chain())
        .run();
}

fn setup() {
    /* Setup camera and other stuff */
}

fn load_texture_atlas(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(
        SpriteSheetBundle {
            texture_atlas: asset_server.load("example.titan"),
            ..default()
        }
    );
}
```

## Documentation

[Full API Documentation](https://docs.rs/bevy_titan)

[File format specifiction](https://github.com/KirmesBude/bevy_titan/blob/main/docs/FileFormatSpecification.md)

[Examples](https://github.com/KirmesBude/bevy_titan/tree/main/examples)

## Future Work

* Make use of bevy's AssetProcessor.

## License

bevy_titan is free, open source and permissively licensed!
Except where noted (below and/or in individual files), all code in this repository is dual-licensed under either:

* MIT License ([LICENSE-MIT](https://github.com/KirmesBude/bevy_titan/blob/main/LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/KirmesBude/bevy_titan/blob/main/LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
This means you can select the license you prefer!

Some of the code was adapted from other sources.
The [assets](https://github.com/KirmesBude/bevy_titan/tree/main/assets) included in this repository fall under different open licenses.
See [CREDITS.md](https://github.com/KirmesBude/bevy_titan/blob/main/CREDITS.md) for the details of the origin of the adapted code and licenses of those files.

### Your contributions

Unless you explicitly state otherwise,
any contribution intentionally submitted for inclusion in the work by you,
as defined in the Apache-2.0 license,
shall be dual licensed as above,
without any additional terms or conditions.
