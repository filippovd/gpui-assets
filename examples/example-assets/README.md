# example-assets

A minimal GPUI application that demonstrates how to register multiple prefixed asset sources and render icons from each one.

## What it shows

- `AssetsRegistry` wired with `LucideAssets`, `MdiAssets`, and `gpui_component_assets::Assets` as the fallback.
- Icons rendered side-by-side from three sources:
  - `gpui-component` built-in icons (`IconName`).
  - `gpui-lucide` (`LucideIcon`).
  - `gpui-mdi` (`MdiIcon`).
- A simple `Button` to show a basic gpui-component interaction.

## Run

```bash
cargo run -p example-assets
```

## Key code

```rust
let assets = gpui_assets::AssetsRegistry::new()
    .use_source(LucideAssets)
    .use_source(MdiAssets)
    .fallback(gpui_component_assets::Assets);

let app = gpui_platform::application().with_assets(assets);
```

See `src/main.rs` for the full window setup and icon grid rendering.
