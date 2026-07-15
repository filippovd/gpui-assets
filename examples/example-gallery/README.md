# example-gallery

A searchable, filterable icon gallery that showcases every icon from the registered asset sources.

## Features

- **Source filter** — switch between Lucide, Material Design Icons, and the gpui-component fallback.
- **Live search** — fuzzy-ish filtering by variant name with debounced background processing.
- **Size selector** — Small / Medium / Large grid cells.
- **Theme selector** — Light, Dark, or System appearance.
- **Info panel** — click an icon to view its variant name, prefixed path, raw SVG, and download the file.
- **Virtualized grid** — large icon sets are rendered with `uniform_list` for efficient scrolling.

## Run

```bash
cargo run -p example-gallery
```

## Structure

- `src/main.rs` — Application setup, asset registry, and window creation.
- `src/gallery.rs` — Main gallery view: search input, filters, virtualized icon grid, status bar.
- `src/icons.rs` — Icon index construction and filtering across all sources.
- `src/search_syntax_popover.rs` — Help popover for the search syntax.

## Key code

```rust
let assets = gpui_assets::AssetsRegistry::new()
    .use_source(LucideAssets)
    .use_source(MdiAssets)
    .fallback(gpui_component_assets::Assets);

let app = gpui_platform::application().with_assets(assets);
```

The gallery builds the full icon index on a background thread so the window opens immediately, then re-runs filtering off the UI thread as the user types.
