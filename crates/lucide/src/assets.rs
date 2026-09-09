use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};
use gpui_assets::PrefixedAssetSource;
use rust_embed::RustEmbed;

/// The asset prefix used by [`LucideAssets`] and the generated [`LucideIcon`] paths.
pub const LUCIDE_PREFIX: &str = "lucide";

/// An [`AssetSource`] that serves bundled Lucide icons via [`RustEmbed`].
///
/// Icons are embedded at compile time from `assets/lucide/icons` (see the
/// `#[folder]` attribute below). Paths passed to [`AssetSource::load`] and
/// [`AssetSource::list`] are resolved against the embedded file names. A leading
/// `/` is stripped to remain compatible with prefix-based routing (e.g.
/// `lucide:/check.svg`).
///
/// Example:
///
/// ```ignore
/// use gpui_lucide::LucideAssets;
/// use gpui_assets::AssetsRegistry;
///
/// let assets = AssetsRegistry::new()
///     .use_source(LucideAssets)
///     .fallback(gpui_kit::assets::Assets);
/// ```
#[derive(RustEmbed)]
#[folder = "../../assets/lucide/icons"]
#[include = "*.svg"]
pub struct LucideAssets;

impl LucideAssets {
    fn resolve<'a>(&self, path: &'a str) -> &'a str {
        path.strip_prefix('/').unwrap_or(path)
    }
}

impl PrefixedAssetSource for LucideAssets {
    fn default_prefix() -> &'static str {
        LUCIDE_PREFIX
    }
}

impl AssetSource for LucideAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        Ok(LucideAssets::get(self.resolve(path)).map(|file| file.data))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let prefix = self.resolve(path);

        Ok(LucideAssets::iter()
            .filter(|p| p.starts_with(prefix))
            .map(SharedString::from)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_existing_icon() {
        let data = LucideAssets.load("check.svg").unwrap().unwrap();
        assert!(!data.is_empty());
        assert!(data.starts_with(b"<svg"));
    }

    #[test]
    fn load_icon_with_leading_slash() {
        let data = LucideAssets.load("/check.svg").unwrap().unwrap();
        assert!(!data.is_empty());
    }

    #[test]
    fn load_missing_icon() {
        assert!(LucideAssets.load("missing.svg").unwrap().is_none());
    }

    #[test]
    fn list_icons() {
        let entries = LucideAssets.list("").unwrap();
        assert!(entries.iter().any(|e| e.as_ref() == "check.svg"));
        assert!(entries.iter().any(|e| e.as_ref() == "pin.svg"));
    }

    #[test]
    fn lucide_assets_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<LucideAssets>();
    }
}
