# example-custom-prefix

A minimal GPUI application that defines its own bundled icon sources with custom routing prefixes.

Unlike `example-assets`, which reuses `gpui-lucide::LucideAssets` and `gpui-mdi::MdiAssets`, this example:

1. Keeps a private copy of 20 Lucide and 20 MDI icons under its own `assets/` directory.
2. Defines `CustomLucideAssets` and `CustomMdiAssets` as `RustEmbed`-backed `AssetSource`s.
3. Registers them under the prefixes `custom-lucide` and `custom-mdi`.
4. Generates `CustomLucideIcon` and `CustomMdiIcon` enums with `icon_named!` so the icons can be used directly with `gpui_component::Icon`.

## Layout

```text
examples/example-custom-prefix/
├── Cargo.toml
├── README.md
├── assets/
│   ├── lucide/icons/   # 20 bundled Lucide SVGs
│   └── mdi/icons/      # 20 bundled MDI SVGs
└── src/
    ├── main.rs         # Demo window and application setup
    ├── assets.rs       # Custom AssetSource implementations and prefixes
    └── icons.rs        # Generated icon enums and RenderOnce impls
```

## Run

```bash
cargo run -p example-custom-prefix
```

## Module overview

### `src/assets.rs`

Defines the custom sources:

```rust
#[derive(RustEmbed)]
#[folder = "assets/lucide/icons"]
#[include = "*.svg"]
pub struct CustomLucideAssets;

impl PrefixedAssetSource for CustomLucideAssets {
    fn default_prefix() -> &'static str {
        "custom-lucide"
    }
}

impl AssetSource for CustomLucideAssets { /* ... */ }
```

The same structure is repeated for `CustomMdiAssets` with the `custom-mdi` prefix.

### `src/icons.rs`

Generates the icon enums and wires each one to `gpui_component::Icon`:

```rust
icon_named!(
    CustomLucideIcon,
    "custom-lucide",
    "assets/lucide/icons",
    [Debug, Copy, PartialEq, Eq]
);

impl RenderOnce for CustomLucideIcon {
    fn render(self, _: &mut Window, _cx: &mut App) -> impl IntoElement {
        Icon::new(self)
    }
}
```

### `src/main.rs`

Wires the registry and renders the demo:

```rust
let assets = gpui_assets::AssetsRegistry::new()
    .use_source(CustomLucideAssets)
    .use_source(CustomMdiAssets)
    .fallback(gpui_component_assets::Assets);
```

```rust
Icon::new(CustomLucideIcon::ALargeSmall).size(px(32.0));
Icon::new(CustomMdiIcon::AbTesting).size(px(32.0));
```
