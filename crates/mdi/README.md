# gpui-mdi

Embedded [Material Design Icons](https://pictogrammers.com/library/mdi/) for [Zed GPUI](https://github.com/zed-industries/zed) and [gpui-component](https://github.com/longbridge/gpui-component).

This crate bundles SVG icons at compile time via [`rust-embed`](https://github.com/pyrossh/rust-embed) and exposes them as:

- `MdiAssets` — an `AssetSource` that can be registered with `AssetsRegistry`.
- `MdiIcon` — a generated enum where each variant maps to one bundled SVG.
- `MDI_PREFIX` — the routing prefix (`"mdi"`).

## Usage

### As an icon

```rust
use gpui_component::Icon;
use gpui_mdi::icons::MdiIcon;

Icon::new(MdiIcon::Check);
```

### As an asset source

```rust
use gpui_assets::AssetsRegistry;
use gpui_mdi::MdiAssets;

let assets = AssetsRegistry::new()
    .use_source(MdiAssets)
    .fallback(gpui_component_assets::Assets);
```

Each variant implements `IconNamed`, so `MdiIcon::Check.path()` returns `mdi:/check.svg`. `AssetsRegistry` routes prefixed paths to the corresponding source.

## Adding icons

Drop new `.svg` files into `assets/mdi/icons/`. The `icon_named!` macro regenerates `MdiIcon` at compile time; file stems are converted to `PascalCase` enum variants (`arrow-right.svg` → `MdiIcon::ArrowRight`).

## See also

- `gpui-assets` — the universal `AssetsRegistry` used for prefix routing.
- `gpui-assets-macros` — the `icon_named!` proc-macro that generates `MdiIcon`.
- `gpui-lucide` — the same pattern applied to Lucide icons.
