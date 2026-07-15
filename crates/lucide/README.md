# gpui-lucide

Embedded [Lucide](https://lucide.dev/) icons for [Zed GPUI](https://github.com/zed-industries/zed) and [gpui-component](https://github.com/longbridge/gpui-component).

This crate bundles SVG icons at compile time via [`rust-embed`](https://github.com/pyrossh/rust-embed) and exposes them as:

- `LucideAssets` — an `AssetSource` that can be registered with `AssetsRegistry`.
- `LucideIcon` — a generated enum where each variant maps to one bundled SVG.
- `LUCIDE_PREFIX` — the routing prefix (`"lucide"`).

## Usage

### As an icon

```rust
use gpui_component::Icon;
use gpui_lucide::icons::LucideIcon;

Icon::new(LucideIcon::Check);
```

### As an asset source

```rust
use gpui_assets::AssetsRegistry;
use gpui_lucide::LucideAssets;

let assets = AssetsRegistry::new()
    .use_source(LucideAssets)
    .fallback(gpui_component_assets::Assets);
```

Each variant implements `IconNamed`, so `LucideIcon::Check.path()` returns `lucide:/check.svg`. `AssetsRegistry` routes prefixed paths to the corresponding source.

## Adding icons

Drop new `.svg` files into `assets/lucide/icons/`. The `icon_named!` macro regenerates `LucideIcon` at compile time; file stems are converted to `PascalCase` enum variants (`arrow-right.svg` → `LucideIcon::ArrowRight`).

## See also

- `gpui-assets` — the universal `AssetsRegistry` used for prefix routing.
- `gpui-assets-macros` — the `icon_named!` proc-macro that generates `LucideIcon`.
- `gpui-mdi` — the same pattern applied to Material Design Icons.
