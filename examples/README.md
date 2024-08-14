# Examples

These examples demonstrate the features of `bevy_titan`` and how to use them.
To run an example, use the command `cargo run --example <Example>`.

```sh
cargo run --example homogeneous_sprite_sheet
```

---

Example                        | Description |
-------------------------------|-------------|
[Homogeneous sprite sheet]     | Shows of how to use `bevy_titan` for homogeneous sprite sheets. |
[Heterogeneous sprite sheet]   | Shows of how to use `bevy_titan` for heterogeneous sprite sheets. |
[Composite texture atlas]      | Shows of how to use `bevy_titan` to create a texture atlas from multiple images. |
[Using bevy_asset_loader]      | Simple example with [bevy_asset_loader]. |
[Exploring TitanConfiguration] | Shows of how to use `bevy_titan`'s configuration to change how the asset is loaded. |

[Homogeneous sprite sheet]: ../examples/homogeneous_sprite_sheet.rs
[Heterogeneous sprite sheet]: ../examples/heterogeneous_sprite_sheet.rs
[Composite texture atlas]: ../examples/composite_texture_atlas.rs
[Using bevy_asset_loader]: ../examples/bevy_asset_loader.rs
[Exploring TitanConfiguration]: ../examples/titan_configuration.rs
[bevy_asset_loader]: https://crates.io/crates/bevy_asset_loader
