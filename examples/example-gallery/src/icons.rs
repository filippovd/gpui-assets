//! Icon data model and index construction.
//!
//! [`all_icons`] builds the full list of [`IconEntry`] from every registered
//! asset source, and [`filter_icons`] ranks them against a search query using
//! the `nucleo-matcher` fuzzy scorer.

use std::sync::Arc;

use gpui::SharedString;
use gpui_assets_utils::pascal_case_name;
use gpui_lucide::{LUCIDE_PREFIX, LucideAssets};
use gpui_mdi::{MDI_PREFIX, MdiAssets};
use nucleo_matcher::{
    Matcher, Utf32String,
    pattern::{CaseMatching, Normalization, Pattern},
};

/// State of the icon index. Built off the UI thread so window opening is not
/// blocked by the ~9k-entry construction.
pub enum IndexState {
    /// Index is still being built by a background task.
    Loading,
    /// Index is ready; cloning the `Arc` only bumps a refcount.
    Ready(Arc<Vec<IconEntry>>),
}

/// Which asset source an icon belongs to (or all of them at once).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IconSource {
    All,
    Mdi,
    Lucide,
    Fallback,
}

impl IconSource {
    pub const ALL: [IconSource; 4] = [
        IconSource::All,
        IconSource::Mdi,
        IconSource::Lucide,
        IconSource::Fallback,
    ];

    pub fn label(self) -> &'static str {
        match self {
            IconSource::All => "All",
            IconSource::Mdi => "Mdi",
            IconSource::Lucide => "Lucide",
            IconSource::Fallback => "Fallback",
        }
    }
}

/// A single icon in the gallery index.
#[derive(Clone)]
pub struct IconEntry {
    pub path: SharedString,
    pub source: IconSource,
    pub variant_name: SharedString,
    /// The searchable segment of `variant_name` (the part after `::`),
    /// stored as UTF-32 so it can be passed directly to `nucleo-matcher`
    /// without per-query allocation.
    pub search_name: Utf32String,
}

/// Build the full icon index from every registered source. Called once per
/// `IconGallery` (in `new`), not per render.
pub fn all_icons() -> Vec<IconEntry> {
    let mut icons = Vec::new();

    for name in LucideAssets::iter() {
        let filename = name.as_ref();
        let variant_name = pascal_case_name(filename);
        icons.push(IconEntry {
            path: format!("{}:/{filename}", LUCIDE_PREFIX).into(),
            source: IconSource::Lucide,
            search_name: Utf32String::from(variant_name.as_str()),
            variant_name: format!("LucideIcon::{variant_name}").into(),
        });
    }

    for name in MdiAssets::iter() {
        let filename = name.as_ref();
        let variant_name = pascal_case_name(filename);
        icons.push(IconEntry {
            path: format!("{}:/{filename}", MDI_PREFIX).into(),
            source: IconSource::Mdi,
            search_name: Utf32String::from(variant_name.as_str()),
            variant_name: format!("MdiIcon::{variant_name}").into(),
        });
    }

    for path in gpui_component_assets::Assets::iter() {
        let path_str = path.as_ref();
        let filename = path_str.rsplit('/').next().unwrap_or(path_str);
        let variant_name = pascal_case_name(filename);
        icons.push(IconEntry {
            path: path_str.into(),
            source: IconSource::Fallback,
            search_name: Utf32String::from(variant_name.as_str()),
            variant_name: format!("IconName::{variant_name}").into(),
        });
    }

    icons
}

/// Pure filter pass run on a background worker: restrict to the selected
/// source, then fuzzy-match the query against each variant name's final
/// segment using `nucleo-matcher`. Results are ranked by score in descending
/// order; ties keep their original (alphabetical) order via a stable sort.
///
/// The empty query matches everything with score 0, so the unfiltered
/// alphabetical order is preserved.
///
/// Returns the ranked icons together with `source_total` — the number of
/// icons belonging to the selected `source` *before* the query is applied.
/// This is the "total" denominator shown in the UI; it depends only on the
/// source, not on the query, so the caller caches it instead of recomputing
/// per render.
pub fn filter_icons(
    icons: Arc<Vec<IconEntry>>,
    source: IconSource,
    query: &str,
) -> (Vec<IconEntry>, usize) {
    // `Matcher` owns reusable scratch memory; creating it once per filter pass
    // is cheap enough for a background task and keeps the API self-contained.
    let mut matcher = Matcher::new(nucleo_matcher::Config::DEFAULT);
    let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);

    // Single pass over the index: restrict to the source (counting every such
    // entry for `source_total`) and, for those, compute the fuzzy score.
    // We iterate over the shared index and clone only the matching entries,
    // avoiding a full clone of ~9k items on every keystroke.
    let mut source_total = 0usize;
    let mut scored: Vec<(u32, IconEntry)> = icons
        .iter()
        .filter(|entry| source == IconSource::All || entry.source == source)
        .inspect(|_| source_total += 1)
        .filter_map(|entry| {
            pattern
                .score(entry.search_name.slice(..), &mut matcher)
                .map(|score| (score, entry.clone()))
        })
        .collect();

    // Stable sort by descending score: equal scores retain input order.
    scored.sort_by_key(|b| std::cmp::Reverse(b.0));

    let ranked = scored.into_iter().map(|(_, entry)| entry).collect();
    (ranked, source_total)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(search_name: &str, source: IconSource) -> IconEntry {
        IconEntry {
            path: search_name.into(),
            source,
            search_name: Utf32String::from(search_name),
            variant_name: search_name.into(),
        }
    }

    #[test]
    fn empty_query_matches_all_and_preserves_order() {
        let icons = vec![
            entry("ArrowRight", IconSource::Lucide),
            entry("Check", IconSource::Lucide),
            entry("ArrowLeft", IconSource::Lucide),
        ];
        let (filtered, total) = filter_icons(Arc::new(icons), IconSource::All, "");
        assert_eq!(filtered.len(), 3);
        assert_eq!(total, 3);
        assert_eq!(filtered[0].variant_name.as_ref(), "ArrowRight");
        assert_eq!(filtered[1].variant_name.as_ref(), "Check");
        assert_eq!(filtered[2].variant_name.as_ref(), "ArrowLeft");
    }

    #[test]
    fn source_filter_counts_total_before_query() {
        let icons = vec![
            entry("ArrowRight", IconSource::Lucide),
            entry("Check", IconSource::Mdi),
            entry("ArrowLeft", IconSource::Lucide),
        ];
        let (filtered, total) = filter_icons(Arc::new(icons), IconSource::Lucide, "Arrow");
        assert_eq!(total, 2);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn exact_match_outranks_fuzzy() {
        let icons = vec![
            entry("ArrowRight", IconSource::Lucide),
            entry("RightArrow", IconSource::Lucide),
            entry("ArrowLeft", IconSource::Lucide),
        ];
        let (filtered, _) = filter_icons(Arc::new(icons), IconSource::All, "ArrowRight");
        assert_eq!(
            filtered.first().unwrap().variant_name.as_ref(),
            "ArrowRight"
        );
    }

    #[test]
    fn prefix_anchor_requires_start() {
        let icons = vec![
            entry("ArrowRight", IconSource::Lucide),
            entry("RightArrow", IconSource::Lucide),
        ];
        let (filtered, _) = filter_icons(Arc::new(icons), IconSource::All, "^Arrow");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].variant_name.as_ref(), "ArrowRight");
    }

    #[test]
    fn suffix_anchor_requires_end() {
        let icons = vec![
            entry("ArrowRight", IconSource::Lucide),
            entry("ArrowRightBold", IconSource::Lucide),
        ];
        let (filtered, _) = filter_icons(Arc::new(icons), IconSource::All, "Right$");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].variant_name.as_ref(), "ArrowRight");
    }

    #[test]
    fn exact_substring_requires_contiguous_match() {
        let icons = vec![
            entry("ArrowRight", IconSource::Lucide),
            entry("ArRwL", IconSource::Lucide),
        ];
        let (filtered, _) = filter_icons(Arc::new(icons), IconSource::All, "'rowR");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].variant_name.as_ref(), "ArrowRight");
    }

    #[test]
    fn negation_excludes_matches() {
        let icons = vec![
            entry("ArrowRight", IconSource::Lucide),
            entry("ArrowLeft", IconSource::Lucide),
            entry("Check", IconSource::Lucide),
        ];
        let (filtered, _) = filter_icons(Arc::new(icons), IconSource::All, "Arrow !Right");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].variant_name.as_ref(), "ArrowLeft");
    }

    #[test]
    fn and_semantics_requires_all_atoms() {
        let icons = vec![
            entry("ArrowRightCircle", IconSource::Lucide),
            entry("ArrowRight", IconSource::Lucide),
            entry("Circle", IconSource::Lucide),
        ];
        let (filtered, _) = filter_icons(Arc::new(icons), IconSource::All, "Arrow Circle");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].variant_name.as_ref(), "ArrowRightCircle");
    }

    #[test]
    fn pascal_case_name_handles_separators_and_digits() {
        assert_eq!(pascal_case_name("arrow-right.svg"), "ArrowRight");
        assert_eq!(pascal_case_name("some_icon_name.svg"), "SomeIconName");
        assert_eq!(pascal_case_name("icon-123.svg"), "Icon123");
        assert_eq!(pascal_case_name("a--b__c.d.svg"), "ABCD");
        assert_eq!(pascal_case_name("123icon.svg"), "123icon");
    }

    #[test]
    fn all_icons_is_not_empty_and_has_expected_sources() {
        let icons = all_icons();
        assert!(!icons.is_empty(), "expected a non-empty icon index");

        let has_lucide = icons.iter().any(|e| e.source == IconSource::Lucide);
        let has_mdi = icons.iter().any(|e| e.source == IconSource::Mdi);
        let has_fallback = icons.iter().any(|e| e.source == IconSource::Fallback);
        assert!(has_lucide, "expected at least one Lucide icon");
        assert!(has_mdi, "expected at least one MDI icon");
        assert!(has_fallback, "expected at least one fallback icon");

        let check = icons
            .iter()
            .find(|e| e.variant_name.as_ref() == "LucideIcon::Check");
        assert!(check.is_some(), "expected LucideIcon::Check in the index");
    }
}
