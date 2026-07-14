use std::borrow::Cow;
use std::collections::HashMap;

use gpui::{AssetSource, Result, SharedString};

/// An asset source that knows its own routing prefix.
///
/// Sources that implement this trait can be registered with a single call to
/// [`AssetsRegistry::use_source`] instead of repeating the prefix constant at
/// every call site.
///
/// Example:
///
/// ```ignore
/// use gpui_assets::{AssetsRegistry, PrefixedAssetSource};
/// use gpui_lucide::LucideAssets;
///
/// let registry = AssetsRegistry::new()
///     .use_source(LucideAssets)
///     .fallback(gpui_component_assets::Assets);
/// ```
pub trait PrefixedAssetSource: AssetSource {
    /// The default prefix used when the source is registered with
    /// [`AssetsRegistry::use_source`].
    fn default_prefix() -> &'static str;
}

/// A universal registry of asset sources with prefix routing and a global fallback.
///
/// Paths passed to [`AssetSource::load`] or [`AssetSource::list`] are inspected for
/// a prefix of the form `prefix:rest`. If a source has been registered for that
/// prefix, the request is forwarded with the prefix stripped. Otherwise the global
/// fallback source is used, if any.
///
/// Example:
///
/// ```ignore
/// use gpui_assets::AssetsRegistry;
///
/// let registry = AssetsRegistry::new()
///     .use_prefix("lucide", lucide_source)
///     .fallback(gpui_component_assets::Assets);
///
/// // Routed to the "lucide" source as "icon.svg".
/// registry.load("lucide:icon.svg");
///
/// // Routed to the fallback source as "themes/light.json".
/// registry.load("themes/light.json");
/// ```
#[derive(Default)]
pub struct AssetsRegistry {
    sources: HashMap<String, Box<dyn AssetSource>>,
    fallback: Option<Box<dyn AssetSource>>,
}

impl AssetsRegistry {
    /// Creates an empty registry with no registered sources and no fallback.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an asset source that provides its own prefix.
    ///
    /// This is a shortcut for [`Self::use_prefix`] when the source implements
    /// [`PrefixedAssetSource`].
    ///
    /// This method consumes the registry; use [`Self::add_source`] for a mutable
    /// variant.
    pub fn use_source<T>(mut self, source: T) -> Self
    where
        T: PrefixedAssetSource + 'static,
    {
        self.add_source(source);
        self
    }

    /// Mutable variant of [`Self::use_source`].
    pub fn add_source<T>(&mut self, source: T) -> &mut Self
    where
        T: PrefixedAssetSource + 'static,
    {
        self.add_prefix(T::default_prefix(), source)
    }

    /// Registers an asset source for the given prefix.
    ///
    /// The prefix is normalized to end with a colon, so both `"lucide"` and
    /// `"lucide:"` are accepted.
    ///
    /// This method consumes the registry; use [`Self::add_prefix`] for a mutable
    /// variant.
    pub fn use_prefix(mut self, prefix: impl Into<String>, source: impl AssetSource) -> Self {
        self.add_prefix(prefix, source);
        self
    }

    /// Mutable variant of [`Self::use_prefix`].
    pub fn add_prefix(&mut self, prefix: impl Into<String>, source: impl AssetSource) -> &mut Self {
        let prefix = normalize_prefix(prefix.into());
        self.sources.insert(prefix, Box::new(source));
        self
    }

    /// Sets the global fallback source used when no matching prefix is found.
    ///
    /// This method consumes the registry; use [`Self::set_fallback`] for a mutable
    /// variant.
    pub fn fallback(mut self, source: impl AssetSource) -> Self {
        self.set_fallback(source);
        self
    }

    /// Mutable variant of [`Self::fallback`].
    pub fn set_fallback(&mut self, source: impl AssetSource) -> &mut Self {
        self.fallback = Some(Box::new(source));
        self
    }

    /// Returns the set of registered prefixes.
    pub fn prefixes(&self) -> impl Iterator<Item = &str> {
        self.sources.keys().map(|s| s.as_str())
    }

    /// Returns `true` if a fallback source has been configured.
    pub fn has_fallback(&self) -> bool {
        self.fallback.is_some()
    }

    fn resolve<'a>(&'a self, path: &'a str) -> (Option<&'a dyn AssetSource>, &'a str) {
        let (prefix, rest) = split_prefix(path);

        if let Some(prefix) = prefix
            && let Some(source) = self.sources.get(prefix)
        {
            return (Some(source.as_ref()), rest);
        }

        (self.fallback.as_deref(), path)
    }
}

impl AssetSource for AssetsRegistry {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let (source, rest) = self.resolve(path);
        match source {
            Some(source) => source.load(rest),
            None => Ok(None),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let (source, rest) = self.resolve(path);
        match source {
            Some(source) => source.list(rest),
            None => Ok(vec![]),
        }
    }
}

impl std::fmt::Debug for AssetsRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetsRegistry")
            .field("sources", &self.sources.keys().collect::<Vec<_>>())
            .field("fallback", &self.fallback.is_some())
            .finish()
    }
}

fn normalize_prefix(mut prefix: String) -> String {
    if !prefix.ends_with(':') {
        prefix.push(':');
    }
    prefix
}

fn split_prefix(path: &str) -> (Option<&str>, &str) {
    match path.find(':') {
        Some(idx) => (Some(&path[..idx + 1]), &path[idx + 1..]),
        None => (None, path),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct MockSource {
        name: &'static str,
    }

    impl PrefixedAssetSource for MockSource {
        fn default_prefix() -> &'static str {
            "mock"
        }
    }

    impl AssetSource for MockSource {
        fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
            let data = format!("{}:{}", self.name, path);
            Ok(Some(Cow::Owned(data.into_bytes())))
        }

        fn list(&self, path: &str) -> Result<Vec<SharedString>> {
            Ok(vec![SharedString::from(format!("{}:{}", self.name, path))])
        }
    }

    #[test]
    fn prefix_routing_load_and_list() {
        let registry = AssetsRegistry::new()
            .use_prefix("lucide", MockSource { name: "lucide" })
            .use_prefix("custom:", MockSource { name: "custom" });

        assert_eq!(
            registry.load("lucide:icon.svg").unwrap().unwrap().as_ref(),
            b"lucide:icon.svg"
        );
        assert_eq!(
            registry.load("custom:file.png").unwrap().unwrap().as_ref(),
            b"custom:file.png"
        );

        let entries = registry.list("lucide:icons").unwrap();
        assert!(entries.iter().any(|e| e.as_ref() == "lucide:icons"));
    }

    #[test]
    fn self_registering_source_uses_default_prefix() {
        let registry = AssetsRegistry::new().use_source(MockSource { name: "mock" });

        assert_eq!(
            registry.load("mock:thing.txt").unwrap().unwrap().as_ref(),
            b"mock:thing.txt"
        );

        let prefixes: Vec<_> = registry.prefixes().collect();
        assert_eq!(prefixes, vec!["mock:"]);
    }

    #[test]
    fn add_source_is_mutable_variant() {
        let mut registry = AssetsRegistry::new();
        registry.add_source(MockSource { name: "mock" });

        assert_eq!(
            registry.load("mock:thing.txt").unwrap().unwrap().as_ref(),
            b"mock:thing.txt"
        );
    }

    #[test]
    fn fallback_is_used_when_prefix_missing_or_unknown() {
        let mut registry = AssetsRegistry::new();
        registry
            .add_prefix("lucide", MockSource { name: "lucide" })
            .set_fallback(MockSource { name: "fallback" });

        // No prefix at all.
        assert_eq!(
            registry.load("plain.txt").unwrap().unwrap().as_ref(),
            b"fallback:plain.txt"
        );

        // Unknown prefix — fallback receives the full path.
        assert_eq!(
            registry
                .load("unknown:thing.txt")
                .unwrap()
                .unwrap()
                .as_ref(),
            b"fallback:unknown:thing.txt"
        );

        // Known prefix still routes to the registered source.
        assert_eq!(
            registry.load("lucide:x.svg").unwrap().unwrap().as_ref(),
            b"lucide:x.svg"
        );
    }

    #[test]
    fn no_source_returns_empty() {
        let registry = AssetsRegistry::new();
        assert_eq!(registry.load("anything").unwrap(), None);
        assert!(registry.list("anything").unwrap().is_empty());
    }

    #[test]
    fn prefixes_are_normalized() {
        let registry = AssetsRegistry::new().use_prefix("lucide", MockSource { name: "lucide" });
        let prefixes: Vec<_> = registry.prefixes().collect();
        assert_eq!(prefixes, vec!["lucide:"]);
    }

    #[test]
    fn debug_repr() {
        let registry = AssetsRegistry::new()
            .use_prefix("lucide", MockSource { name: "lucide" })
            .fallback(MockSource { name: "fallback" });

        let repr = format!("{:?}", registry);
        assert!(repr.contains("lucide:"));
        assert!(repr.contains("fallback: true"));
    }

    #[test]
    fn registry_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<AssetsRegistry>();
    }
}
