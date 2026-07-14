# AGENTS.md — Working with this project

## Project Overview

This is a Rust Cargo workspace that builds desktop applications with Zed's `gpui` and the `gpui-component` UI library. The workspace focuses on **asset management**: a prefix-routing registry, a proc-macro for icon enums, and an embedded Lucide icon set.

## Workspace Layout

- `crates/assets` — `AssetsRegistry`, universal `AssetSource` registry (crate name `gpui-assets`).
- `crates/macros` — `icon_named!` proc-macro (crate name `gpui-assets-macros`).
- `crates/lucide` — `LucideAssets` (RustEmbed source) + generated `LucideIcon` enum (crate name `gpui-lucide`).
- `crates/mdi` — `MdiAssets` (RustEmbed source) + generated `MdiIcon` enum (crate name `gpui-mdi`).
- `examples/example-assets` — runnable example.
- `examples/example-gallery` — icon gallery with source filtering (Lucide, MDI, fallback), search, and theme/size controls.

All crates list dependencies from `[workspace.dependencies]` in the root `Cargo.toml`.

## Build & Test Commands

```bash
# Check the whole workspace
cargo check --workspace

# Run all tests
cargo test --workspace

# Run a specific crate's tests
cargo test -p gpui-lucide

# Run the examples
cargo run -p example-assets
cargo run -p example-gallery
```

> **Windows note:** Real-time antivirus scanners may lock freshly-built test binaries, causing `cargo test` to fail with access-denied or "file in use" errors. Use an alternate target directory when this happens:
> ```bash
> CARGO_TARGET_DIR=/tmp/gpui-test-target cargo test -p gpui-lucide
> ```

## Conventions

### Dependencies

- Declare all dependencies in the **root** `Cargo.toml` under `[workspace.dependencies]`.
- Member crates reference them with `{ workspace = true }`.
- Do not add direct versioned dependencies in member `Cargo.toml` files unless there is a strong reason.

### Code Style

- Use `edition = "2024"` (declared in `[workspace.package]`).
- Keep crate APIs minimal and focused.
- Prefer explicit imports over glob imports in library code; glob imports are acceptable in examples.
- Import `SharedString` and `IntoElement` from `gpui`, not from `gpui_component`.
- `IconNamed` is imported from `gpui_component`.

### Asset Paths

- `icon_named!` generates paths in the form `{prefix}:/{filename}` (e.g. `lucide:/check.svg`).
- `AssetsRegistry::split_prefix` splits at the first `:` and treats the prefix as including the colon (`lucide:`).
- `AssetSource` implementations should strip a leading `/` from the remaining path to stay cross-platform friendly.
- In `gpui-lucide`, use the exported `LUCIDE_PREFIX` constant instead of hard-coding `"lucide"`.
- In `gpui-mdi`, use the exported `MDI_PREFIX` constant instead of hard-coding `"mdi"`.

### Adding a New Crate

1. Create the crate under `crates/<crate-name>`.
2. Add `"crates/<crate-name>"` to `[workspace.members]` in the root `Cargo.toml`.
3. Use `version.workspace = true`, `edition.workspace = true`, etc. in the crate's `Cargo.toml`.
4. Add any new shared dependencies to `[workspace.dependencies]`.
5. Run `cargo check --workspace`.

### Adding a New Prefixed Asset Source

This workspace uses one crate per bundled asset source. `gpui-lucide` and `gpui-mdi` are the reference implementations. To add a new source (e.g. `heroicons`) follow the same pattern.

#### 1. Add the assets

Create a directory for the raw assets. For a flat icon set the convention is `assets/<name>/icons/`:

```text
assets/heroicons/icons/
├── arrow-right.svg
├── check.svg
└── ...
```

Only the file names matter for a flat set; `RustEmbed` will embed them with those names.

#### 2. Create the crate

```bash
mkdir -p crates/heroicons/src
```

`crates/heroicons/Cargo.toml`:

```toml
[package]
name = "gpui-heroicons"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true

[dependencies]
gpui-assets-macros = { path = "../macros" }

gpui = { workspace = true }
gpui-component = { workspace = true }
rust-embed = { workspace = true }
```

Add `"crates/heroicons"` to `[workspace.members]` in the root `Cargo.toml`.

#### 3. Implement `AssetSource`

`crates/heroicons/src/assets.rs`:

```rust
use std::borrow::Cow;
use std::path::PathBuf;

use gpui::{AssetSource, Result, SharedString};
use rust_embed::RustEmbed;

pub const HEROICONS_PREFIX: &str = "heroicons";

/// Embedded Heroicons asset source.
#[derive(RustEmbed)]
#[folder = "../../assets/heroicons/icons"]
#[include = "*.svg"]
pub struct HeroiconsAssets;

impl HeroiconsAssets {
    fn resolve(&self, path: &str) -> PathBuf {
        // `AssetsRegistry` strips the prefix, leaving a leading `/` on the
        // path produced by `icon_named!`. Strip it so RustEmbed can look up
        // the file by its bare name.
        let path = path.strip_prefix('/').unwrap_or(path);
        PathBuf::from(path)
    }
}

impl AssetSource for HeroiconsAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let full_path = self.resolve(path);
        Ok(HeroiconsAssets::get(&full_path.to_string_lossy()).map(|file| file.data))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let prefix = self.resolve(path);
        let prefix = prefix.to_string_lossy();

        Ok(HeroiconsAssets::iter()
            .filter(|p| p.starts_with(prefix.as_ref()))
            .map(SharedString::from)
            .collect())
    }
}
```

The prefix constant is exposed so callers never hard-code the string.

#### 4. Generate the icon enum

`crates/heroicons/src/icons.rs`:

```rust
use gpui::{IntoElement, RenderOnce, SharedString};
use gpui_assets_macros::icon_named;
use gpui_component::IconNamed;

icon_named!(
    HeroiconsIcon,
    "heroicons",
    "../../assets/heroicons/icons",
    [Debug, Copy, PartialEq, Eq]
);

impl RenderOnce for HeroiconsIcon {
    fn render(self, _: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        gpui_component::Icon::new(self)
    }
}
```

`icon_named!` takes:

1. The enum name (`HeroiconsIcon`).
2. The prefix (`"heroicons"`) — must match `HEROICONS_PREFIX`.
3. The relative path to the folder of SVGs.
4. Derives to attach to the enum.

It produces one variant per `.svg` file. Variant names are `PascalCase` conversions of the file stem, so `arrow-right.svg` becomes `HeroiconsIcon::ArrowRight`. Each variant implements `IconNamed`, which provides a `path()` method returning `heroicons:/arrow-right.svg`.

#### 5. Wire up the crate

`crates/heroicons/src/lib.rs`:

```rust
//! GPUI asset source for Heroicons.

mod assets;

pub mod icons;

pub use assets::{HEROICONS_PREFIX, HeroiconsAssets};
```

#### 6. Implement `PrefixedAssetSource`

Implement the `PrefixedAssetSource` trait from `gpui-assets` so the crate can register itself by prefix:

```rust
use gpui_assets::PrefixedAssetSource;

impl PrefixedAssetSource for HeroiconsAssets {
    fn default_prefix() -> &'static str {
        HEROICONS_PREFIX
    }
}
```

#### 7. Register the source

In your application, combine the new source with the registry and any others. Because the source now provides its own prefix, use `use_source` instead of `use_prefix`:

```rust
use gpui_assets::AssetsRegistry;
use gpui_heroicons::HeroiconsAssets;
use gpui_lucide::LucideAssets;
use gpui_mdi::MdiAssets;

let assets = AssetsRegistry::new()
    .use_source(HeroiconsAssets)
    .use_source(LucideAssets)
    .use_source(MdiAssets)
    .fallback(gpui_component_assets::Assets);
```

`use_source` is equivalent to `.use_prefix(HEROICONS_PREFIX, HeroiconsAssets)`; the explicit form is still available for custom or override prefixes.

#### 8. Use the icons

```rust
use gpui_component::Icon;
use gpui_heroicons::icons::HeroiconsIcon;

Icon::new(HeroiconsIcon::ArrowRight);
```

#### Important conventions

- The embedded folder path in `#[folder]` and the path passed to `icon_named!` must point to the same directory. Both are relative to `crates/<name>/src/`.
- `AssetSource::load`/`list` must strip a leading `/` from the path. `icon_named!` produces `{prefix}:/{filename}`, and `AssetsRegistry` forwards `/{filename}` to the source.
- Every bundled source should implement `PrefixedAssetSource` and return the same prefix used by `icon_named!`.
- Keep the crate API minimal: re-export the prefix constant, the `AssetSource` struct, and the `icons` module.

### Adding Icons

1. Add an `.svg` file to `assets/lucide/icons/`, `assets/mdi/icons/`, or the corresponding folder for a custom source.
2. The `icon_named!` macro will generate a new variant such as `LucideIcon::{PascalCaseName}` or `HeroiconsIcon::{PascalCaseName}` at compile time.
3. Update any tests that enumerate expected variants.

### Proc-Macro Crate

- `gpui-assets-macros` is a `proc-macro = true` crate.
- Keep it dependency-light: only `proc-macro2`, `quote`, `syn`.
- The generated code relies on `IconNamed`, `SharedString`, and `IntoElement` being in scope at the call site.

## Common Pitfalls

- `#[derive(IntoElement)]` requires the type to implement `RenderOnce`. When using `icon_named!`, provide a manual `RenderOnce` impl if the enum needs to be rendered directly.
- `gpui_component::SharedString` is private; always import `SharedString` from `gpui`.
- Do not derive `Clone` manually when `icon_named!` already derives it.
- `RustEmbed` paths must match the paths produced by `icon_named!` after prefix stripping. In `gpui-lucide` the embedded folder is `assets/icons` and included files are flat (`*.svg`), so embedded paths are just file names.

## When to Ask the User

- Before introducing new external dependencies not already in the workspace.
- Before changing the public API of `gpui-assets` (e.g. renaming `use_prefix`/`fallback`).
- Before adding binary/example crates beyond `examples/example-assets`.

## Skills Reference

This project has custom skills to assist with common development tasks:

- **gpui** (`skills/`) - GPUI framework knowledge: actions/keybindings, async, context, custom elements, entity state, events, focus, global state, layout/styling, testing
- **gpui-component** (`skills/`) - How to use gpui-component: setup, stateless/stateful patterns, common component APIs, theming

When working on tasks related to these areas, agents will automatically use the appropriate skill to provide specialized guidance and patterns.
