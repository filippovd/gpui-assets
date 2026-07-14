# Assets

This directory holds bundled icon sets used by the workspace crates.

## Layout

```
assets/
├── lucide/icons/   # Lucide icon SVGs (used by crates/lucide)
├── mdi/icons/      # Material Design Icon SVGs (used by crates/mdi)
├── .gitignore      # Ignores the icon subdirectories below
└── README.md       # This file
```

## Icon sets

- **`lucide/icons/`** — SVG icons consumed by the `gpui-lucide` crate. The
  `icon_named!` macro scans this folder at compile time and generates the
  `LucideIcon` enum.
- **`mdi/icons/`** — SVG icons consumed by the `gpui-mdi` crate. The
  `icon_named!` macro scans this folder at compile time and generates the
  `MdiIcon` enum.

## Crate references

Each icon crate embeds its folder via `RustEmbed`, using a relative path from
its `src/` directory:

| Crate | Embedded folder |
|-------|-----------------|
| `crates/lucide` | `../../assets/lucide/icons` |
| `crates/mdi` | `../../assets/mdi/icons` |

## Adding icons

Drop a new `.svg` file into the appropriate `assets/<set>/icons/` folder. The
matching crate will pick it up automatically on the next build and expose it as
`<Set>Icon::{PascalCaseFileName}`.

## Git tracking

The `lucide/` and `mdi/` subdirectories are listed in `.gitignore` because they
contain large, vendored icon collections. Only the folder structure, this
README, and the ignore rules are tracked in the repository.

## References

- Lucide icons can be sourced from [lucide.dev](https://lucide.dev).
  Reference: https://github.com/lucide-icons/lucide/tree/main/icons
- Material Design icons are sourced from github.com.
  Reference: https://github.com/Templarian/MaterialDesign-SVG
