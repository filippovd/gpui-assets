# example-assets

A minimal GPUI Kit application that demonstrates how to register multiple prefixed asset sources and render icons from each one.

## What it shows

- `AssetsRegistry` wired with `LucideAssets`, `MdiAssets`, and `gpui_kit::assets::Assets` as the fallback.
- Icons rendered side-by-side from three sources:
  - `gpui-kit` built-in icons (`IconName`).
  - `gpui-lucide` (`LucideIcon`).
  - `gpui-mdi` (`MdiIcon`).
- A simple `Button` to show a basic gpui-kit interaction.

## Run

```bash
cargo run -p example-assets
```

## Key code

```rust
let assets = gpui_assets::AssetsRegistry::new()
    .use_source(LucideAssets)
    .use_source(MdiAssets)
    .fallback(gpui_kit::assets::Assets);

let app = gpui_kit::application().with_assets(assets);
```

See `src/main.rs` for the full window setup and icon grid rendering.
