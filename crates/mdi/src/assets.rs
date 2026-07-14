use std::borrow::Cow;
use std::path::PathBuf;

use gpui::{AssetSource, Result, SharedString};
use gpui_assets::PrefixedAssetSource;
use rust_embed::RustEmbed;

/// The asset prefix used by [`MdiAssets`] and the generated [`MdiIcon`] paths.
pub const MDI_PREFIX: &str = "mdi";

/// An [`AssetSource`] that serves bundled Material Design Icons via [`RustEmbed`].
///
/// Icons are embedded at compile time from `assets/mdi/icons` (see the
/// `#[folder]` attribute below). Paths passed to [`AssetSource::load`] and
/// [`AssetSource::list`] are resolved against the embedded file names. A leading
/// `/` is stripped to remain compatible with prefix-based routing (e.g.
/// `mdi:/check.svg`).
///
/// Example:
///
/// ```ignore
/// use gpui_mdi::MdiAssets;
/// use gpui_assets::AssetsRegistry;
///
/// let assets = AssetsRegistry::new()
///     .use_source(MdiAssets)
///     .fallback(gpui_component_assets::Assets);
/// ```
#[derive(RustEmbed)]
#[folder = "../../assets/mdi/icons"]
#[include = "*.svg"]
pub struct MdiAssets;

impl MdiAssets {
    fn resolve(&self, path: &str) -> PathBuf {
        let path = path.strip_prefix('/').unwrap_or(path);
        PathBuf::from(path)
    }
}

impl PrefixedAssetSource for MdiAssets {
    fn default_prefix() -> &'static str {
        MDI_PREFIX
    }
}

impl AssetSource for MdiAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let full_path = self.resolve(path);
        Ok(MdiAssets::get(&full_path.to_string_lossy()).map(|file| file.data))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let prefix = self.resolve(path);
        let prefix = prefix.to_string_lossy();

        Ok(MdiAssets::iter()
            .filter(|p| p.starts_with(prefix.as_ref()))
            .map(SharedString::from)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_existing_icon() {
        let data = MdiAssets.load("check.svg").unwrap().unwrap();
        assert!(!data.is_empty());
        assert!(data.starts_with(b"<svg"));
    }

    #[test]
    fn load_icon_with_leading_slash() {
        let data = MdiAssets.load("/check.svg").unwrap().unwrap();
        assert!(!data.is_empty());
    }

    #[test]
    fn load_missing_icon() {
        assert!(MdiAssets.load("missing.svg").unwrap().is_none());
    }

    #[test]
    fn list_icons() {
        let entries = MdiAssets.list("").unwrap();
        assert!(entries.iter().any(|e| e.as_ref() == "check.svg"));
        assert!(entries.iter().any(|e| e.as_ref() == "pin.svg"));
    }

    #[test]
    fn mdi_assets_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MdiAssets>();
    }
}
