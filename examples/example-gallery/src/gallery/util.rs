//! Small free-standing helpers shared across the gallery: asset-path parsing,
//! platform path discovery, theme mapping, and SVG loading.

use std::path::PathBuf;

use gpui::WindowAppearance;
use gpui_component::ThemeMode;
use gpui_lucide::LucideAssets;
use gpui_mdi::MdiAssets;

use crate::icons::{IconEntry, IconSource};

/// Return the last path segment of an asset path, stripping any prefix.
///
/// Examples:
/// - `lucide:/check.svg` -> `check.svg`
/// - `icons/check.svg`   -> `check.svg`
pub(crate) fn file_name_from_path(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// Return the user's Downloads folder, falling back to the current directory
/// when the folder can't be determined or doesn't exist.
///
/// `USERPROFILE` is checked first because on Windows `HOME` is often a
/// Git Bash/MSYS-injected Unix-style path (e.g. `/c/Users/...`) that the native
/// file dialog can't resolve; `HOME` then covers macOS/Linux.
pub(crate) fn downloads_dir() -> PathBuf {
    let candidate = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .map(|home| home.join("Downloads"))
        .filter(|dir| dir.exists());
    candidate.unwrap_or_else(|| PathBuf::from("."))
}

/// Map a window appearance to a `ThemeMode`.
pub(crate) fn system_theme_mode(appearance: WindowAppearance) -> ThemeMode {
    match appearance {
        WindowAppearance::Dark | WindowAppearance::VibrantDark => ThemeMode::Dark,
        _ => ThemeMode::Light,
    }
}

/// Load the raw SVG text for an icon entry, if it can be read as UTF-8.
pub(crate) fn load_svg_content(entry: &IconEntry) -> Option<String> {
    let bytes = match entry.source {
        IconSource::Lucide => {
            let name = file_name_from_path(entry.path.as_ref());
            LucideAssets::get(name)?.data
        }
        IconSource::Mdi => {
            let name = file_name_from_path(entry.path.as_ref());
            MdiAssets::get(name)?.data
        }
        IconSource::Fallback => gpui_component_assets::Assets::get(entry.path.as_ref())?.data,
        IconSource::All => return None,
    };
    String::from_utf8(bytes.into_owned()).ok()
}
