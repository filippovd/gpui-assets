use std::borrow::Cow;
use std::path::PathBuf;

use gpui::{AssetSource, Result, SharedString};
use gpui_assets::PrefixedAssetSource;
use rust_embed::RustEmbed;

/// Prefix for the custom Lucide source.
pub const CUSTOM_LUCIDE_PREFIX: &str = "custom-lucide";

/// Prefix for the custom MDI source.
pub const CUSTOM_MDI_PREFIX: &str = "custom-mdi";

/// A custom Lucide asset source that embeds icons from the example's own
/// `assets/lucide/icons` directory under the `custom-lucide` prefix.
#[derive(RustEmbed)]
#[folder = "assets/lucide/icons"]
#[include = "*.svg"]
pub struct CustomLucideAssets;

impl CustomLucideAssets {
    fn resolve(&self, path: &str) -> PathBuf {
        let path = path.strip_prefix('/').unwrap_or(path);
        PathBuf::from(path)
    }
}

impl PrefixedAssetSource for CustomLucideAssets {
    fn default_prefix() -> &'static str {
        CUSTOM_LUCIDE_PREFIX
    }
}

impl AssetSource for CustomLucideAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let full_path = self.resolve(path);
        Ok(CustomLucideAssets::get(&full_path.to_string_lossy()).map(|file| file.data))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let prefix = self.resolve(path);
        let prefix = prefix.to_string_lossy();

        Ok(CustomLucideAssets::iter()
            .filter(|p| p.starts_with(prefix.as_ref()))
            .map(SharedString::from)
            .collect())
    }
}

/// A custom MDI asset source that embeds icons from the example's own
/// `assets/mdi/icons` directory under the `custom-mdi` prefix.
#[derive(RustEmbed)]
#[folder = "assets/mdi/icons"]
#[include = "*.svg"]
pub struct CustomMdiAssets;

impl CustomMdiAssets {
    fn resolve(&self, path: &str) -> PathBuf {
        let path = path.strip_prefix('/').unwrap_or(path);
        PathBuf::from(path)
    }
}

impl PrefixedAssetSource for CustomMdiAssets {
    fn default_prefix() -> &'static str {
        CUSTOM_MDI_PREFIX
    }
}

impl AssetSource for CustomMdiAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let full_path = self.resolve(path);
        Ok(CustomMdiAssets::get(&full_path.to_string_lossy()).map(|file| file.data))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let prefix = self.resolve(path);
        let prefix = prefix.to_string_lossy();

        Ok(CustomMdiAssets::iter()
            .filter(|p| p.starts_with(prefix.as_ref()))
            .map(SharedString::from)
            .collect())
    }
}
