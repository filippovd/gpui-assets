//! The icon gallery view: builds the icon index lazily on a background thread,
//! renders a searchable, filterable grid of icon buttons.

mod about;
mod info_panel;
mod util;

use std::cell::RefCell;
use std::ops::Range;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{
    ActiveTheme, Icon, Root, Selectable, Sizable, StyledExt, Theme, ThemeMode, TitleBar,
    button::{Button, ButtonGroup, ButtonVariants},
    input::{Input, InputState},
    radio::RadioGroup,
    scroll::ScrollableElement as _,
    spinner::Spinner,
    status_bar::StatusBar,
};

use crate::icons::{IconSource, IndexState, all_icons, filter_icons};
use crate::search_syntax_popover::SearchSyntaxPopover;
use gpui_mdi::icons::MdiIcon;
use util::system_theme_mode;

/// Debounce window for the search input: a burst of keystrokes only triggers
/// one filter pass after the user pauses for this long.
const DEBOUNCE: Duration = Duration::from_millis(50);

/// Horizontal space taken by the list container's padding and border. This is
/// subtracted from the viewport width so the column count fits the actual
/// available width without reaching into internal scroll-handle state.
const LIST_CONTAINER_INSET: Pixels = px(20.0);

/// Icon grid size selected by the user.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Size {
    Small,
    Medium,
    Large,
}

impl Size {
    const ALL: [Size; 3] = [Size::Small, Size::Medium, Size::Large];

    fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }

    fn row_height(self) -> Pixels {
        match self {
            Size::Small => px(28.0),
            Size::Medium => px(36.0),
            Size::Large => px(44.0),
        }
    }

    fn label(self) -> &'static str {
        match self {
            Size::Small => "Small",
            Size::Medium => "Medium",
            Size::Large => "Large",
        }
    }
}

/// Theme mode selected by the user. The "System" variant follows the OS
/// appearance via `observe_window_appearance`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ThemeModeSelection {
    Light,
    Dark,
    System,
}

impl ThemeModeSelection {
    const ALL: [ThemeModeSelection; 3] = [
        ThemeModeSelection::Light,
        ThemeModeSelection::Dark,
        ThemeModeSelection::System,
    ];

    fn index(self) -> usize {
        self as usize
    }

    fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }

    fn label(self) -> &'static str {
        match self {
            ThemeModeSelection::Light => "Light",
            ThemeModeSelection::Dark => "Dark",
            ThemeModeSelection::System => "System",
        }
    }
}

pub struct IconGallery {
    size: Option<Size>,
    theme: Option<ThemeModeSelection>,
    source_index: Option<usize>,
    scroll_handle: UniformListScrollHandle,
    search_input: Entity<InputState>,
    pub(super) selected_icon: Option<crate::icons::IconEntry>,
    /// Full icon index, built once on a background thread. Filtered views are
    /// derived from this instead of rebuilding ~9k `IconEntry`s on every render.
    icons: IndexState,
    /// Result of the most recent (background) search/filter pass. Read directly
    /// by `render`; updated by `run_search` on the UI thread.
    filtered_icons: Arc<Vec<crate::icons::IconEntry>>,
    /// Total number of icons in the currently selected source, ignoring the
    /// search query. Recomputed by `run_search` only when the source changes,
    /// so `render` never re-scans the ~9k-entry index.
    source_total: usize,
    /// Holds the in-flight index-build task so it is not cancelled prematurely.
    _indexing: Option<Task<()>>,
    /// Debounce + filter task. Replacing it cancels the previous wait, so only
    /// the latest keystroke actually runs the filter.
    _debounce: Option<Task<()>>,
    /// True while a background search/filter pass is in flight; drives the
    /// "Searching…" overlay.
    searching: bool,
    /// Last query/source that `run_search` actually ran, to skip redundant
    /// re-filtering when the input notifies but nothing changed.
    last_query: String,
    last_source: Option<usize>,
    /// Subscription to system appearance changes; only active when the user
    /// selects the "System" theme option.
    _system_appearance: Option<Subscription>,
    /// Standalone search-syntax help popover component.
    search_syntax_popover: Entity<SearchSyntaxPopover>,
    /// Directory used as the default location for the Save As dialog on the
    /// last successful SVG download. Falls back to the user's Downloads folder.
    pub(super) last_save_dir: Option<PathBuf>,
    /// Row range currently visible in the icon grid. Updated by the
    /// `uniform_list` closure during layout and read by the status bar.
    visible_rows: Rc<RefCell<Range<usize>>>,
}

impl IconGallery {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Search by VariantName...")
                .clean_on_escape()
        });
        cx.observe(&search_input, |this, _input, cx| this.run_search(cx))
            .detach();

        // Build the icon index on a background thread so the window can open
        // immediately. The ~9k-entry construction (with its `format!`
        // allocations) would otherwise block the UI for tens of milliseconds.
        let build = cx.background_spawn(async move { all_icons() });
        let indexing = cx.spawn(async move |this, cx: &mut AsyncApp| {
            let result = build.await;
            this.update(cx, |view, cx| {
                view.icons = IndexState::Ready(Arc::new(result));
                // Run the initial filter (empty query → full list, ranked) now
                // that the index is available.
                view.run_search(cx);
                cx.notify();
            })
            .ok();
        });

        // Start in System mode and react to OS appearance changes while it is
        // selected.
        let mode = system_theme_mode(window.appearance());
        Theme::change(mode, Some(window), cx);
        let system_appearance = cx.observe_window_appearance(window, |this, window, cx| {
            if this.theme == Some(ThemeModeSelection::System) {
                let mode = system_theme_mode(window.appearance());
                Theme::change(mode, Some(window), cx);
            }
        });

        let search_syntax_popover = cx.new(SearchSyntaxPopover::new);

        Self {
            size: Some(Size::Medium),
            theme: Some(ThemeModeSelection::System),
            source_index: Some(0),
            scroll_handle: UniformListScrollHandle::new(),
            search_input,
            selected_icon: None,
            icons: IndexState::Loading,
            filtered_icons: Arc::new(Vec::new()),
            source_total: 0,
            _indexing: Some(indexing),
            _debounce: None,
            searching: false,
            last_query: String::new(),
            last_source: None,
            _system_appearance: Some(system_appearance),
            search_syntax_popover,
            last_save_dir: None,
            visible_rows: Rc::new(RefCell::new(0..0)),
        }
    }

    /// Re-run the icon filter (source + search query) on a background thread and
    /// publish the result back on the UI thread.
    ///
    /// The heavy part — scanning ~9k entries, lowercasing names, `contains`
    /// checks — runs off the UI thread via `background_spawn`. Because entities
    /// cannot be updated from a background task, the result is awaited inside a
    /// foreground `spawn` and applied through `this.update`.
    ///
    /// No-ops while the index is still loading; once the index is ready the
    /// completion callback in `new` triggers the initial run.
    fn run_search(&mut self, cx: &mut Context<Self>) {
        // Index not ready yet — the build completion callback will start the
        // initial search once it lands.
        if matches!(self.icons, IndexState::Loading) {
            return;
        }

        let selected_source = IconSource::ALL[self.source_index.unwrap_or(0)];
        let query = self.search_input.read(cx).value().to_string();

        // Skip redundant work: the input notifies on focus/click as well as on
        // text changes, so guard against re-running an identical filter pass.
        if query == self.last_query && Some(self.source_index.unwrap_or(0)) == self.last_source {
            return;
        }
        self.last_query = query.clone();
        self.last_source = Some(self.source_index.unwrap_or(0));

        // Debounce: wait for a short quiet period before running the (heavy)
        // filter pass, so a burst of keystrokes only triggers one scan. Storing
        // the task in `_debounce` cancels any in-flight wait when a new keystroke
        // arrives (dropping the previous Task cancels it).
        let timer = cx.background_executor().timer(DEBOUNCE);
        let task = cx.spawn(async move |this, cx: &mut AsyncApp| {
            // Quiet period — bail out silently if superseded by a newer search.
            timer.await;

            // Read the current index snapshot inside the task so a freshly-loaded
            // index is picked up, then run the filter off the UI thread.
            let all_icons = match this.read_with(cx, |view, _cx| match &view.icons {
                IndexState::Ready(icons) => Some(Arc::clone(icons)),
                IndexState::Loading => None,
            }) {
                Ok(Some(all_icons)) => all_icons,
                _ => return,
            };
            let filter = cx
                .background_spawn(async move { filter_icons(all_icons, selected_source, &query) });
            let (ranked, source_total) = filter.await;
            this.update(cx, |view, cx| {
                view.filtered_icons = Arc::new(ranked);
                view.source_total = source_total;
                view.searching = false;
                cx.notify();
            })
            .ok();
        });
        self._debounce = Some(task);
        // Show the "Searching…" overlay immediately; the background pass clears
        // it on completion.
        self.searching = true;
        cx.notify();
    }

    // `download_svg` and `render_info_panel` live in `info_panel.rs`, as an
    // `impl IconGallery` block that reaches this struct's state directly.
}

impl Render for IconGallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // While the icon index is being built on a background thread, show a
        // placeholder instead of the full gallery. This single early return
        // isolates the loading state without duplicating the UI tree.
        if matches!(self.icons, IndexState::Loading) {
            return div()
                .flex()
                .size_full()
                .items_center()
                .justify_center()
                .child("Loading icons…")
                .into_any_element();
        }

        let selected_size = self.size.unwrap_or(Size::Medium);
        let row_height = selected_size.row_height();
        // Fixed cell width per size tier. Each gallery button is given an
        // explicit width so the column count is deterministic regardless of the
        // button's internal padding (compact/small/large).
        let cell_width = row_height;
        let gap = px(4.0);

        // `source_total` and `filtered_icons` are both recomputed by
        // `run_search` (on a background thread) whenever the source or query
        // changes, so `render` never re-scans the ~9k-entry index.
        let total_count = self.source_total;

        // Derive the column count from the available width. We avoid reading
        // `UniformListScrollHandle`'s internal `RefCell` state during render,
        // because that handle is already borrowed by gpui's layout pass and the
        // field is internal API. The viewport width minus the fixed info panel
        // and container inset gives a stable, safe estimate.
        let container_width =
            (window.viewport_size().width - info_panel::INFO_PANEL_WIDTH - LIST_CONTAINER_INSET)
                .max(px(0.0));
        let columns = if container_width > px(0.0) {
            // N columns need N * cell_width plus (N - 1) gaps to fit.
            ((container_width + gap) / (cell_width + gap)).floor() as usize
        } else {
            1
        };
        let columns = columns.max(1);

        let filtered_count = self.filtered_icons.len();
        let row_count = filtered_count.div_ceil(columns);

        let icon_names_for_list = Arc::clone(&self.filtered_icons);
        let visible_rows = Rc::clone(&self.visible_rows);
        let view = cx.entity();
        let list = uniform_list(
            "icon-list",
            row_count,
            move |visible_range, _window, _cx| {
                *visible_rows.borrow_mut() = visible_range.clone();
                visible_range
                    .map(|row_ix| {
                        let start = row_ix * columns;
                        let end = ((row_ix + 1) * columns).min(icon_names_for_list.len());
                        let row_buttons = icon_names_for_list[start..end].iter().map(|entry| {
                            let icon_path = entry.path.to_string();
                            let tooltip = entry.variant_name.to_string();
                            let view = view.clone();
                            let entry_for_click = entry.clone();
                            let mut btn = Button::new(entry.path.to_string())
                                .icon(Icon::empty().path(icon_path))
                                .ghost()
                                .tooltip(tooltip)
                                .compact()
                                .w(cell_width)
                                .on_click(move |_, _, cx| {
                                    let entry = entry_for_click.clone();
                                    view.update(cx, |view, cx| {
                                        view.selected_icon = Some(entry);
                                        cx.notify();
                                    });
                                });
                            match selected_size {
                                Size::Small => btn = btn.small(),
                                Size::Large => btn = btn.large(),
                                Size::Medium => {}
                            }
                            btn
                        });

                        div()
                            .h_flex()
                            .gap(gap)
                            .items_center()
                            .h(row_height + px(8.0))
                            .children(row_buttons)
                    })
                    .collect()
            },
        )
        .track_scroll(&self.scroll_handle);

        div()
            .relative()
            .size_full()
            .child(
                div()
                    .v_flex()
                    .gap_1()
                    //.p_4()
                    .size_full()
                    .overflow_hidden()
                    .child(
                        // Custom window title bar: the app title sits next to the search
                        // input (the "Title Bar with Search" pattern from gpui-component).
                        // The title bar's inner row is `justify_between`, so a second
                        // child (About) is pushed to the right edge.
                        TitleBar::new()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .child("Icon Gallery")
                                    .child(
                                        div()
                                            .h_flex()
                                            .items_center()
                                            .gap_1()
                                            // Prevent the TitleBar from treating drags inside
                                            // the search input as a window move gesture.
                                            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                                cx.stop_propagation()
                                            })
                                            .child(
                                                Input::new(&self.search_input)
                                                    .w(px(240.0))
                                                    .small()
                                                    .cleanable(true)
                                                    .prefix(
                                                        Icon::new(MdiIcon::ImageSearchOutline)
                                                            .small(),
                                                    )
                                                    .suffix(self.search_syntax_popover.clone()),
                                            ),
                                    ),
                            )
                            .child(about::render_about(cx)),
                    )
                    .child(
                        div()
                            .h_flex()
                            .gap_4()
                            .p_4()
                            .items_center()
                            .child(
                                div()
                                    .h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child("Asset Source:")
                                    .child(
                                        RadioGroup::horizontal("source")
                                            .children(IconSource::ALL.map(|s| s.label()))
                                            .selected_index(self.source_index)
                                            .on_click(cx.listener(|view, index: &usize, _, cx| {
                                                view.source_index = Some(*index);
                                                view.run_search(cx);
                                            })),
                                    ),
                            )
                            .child({
                                // When the search narrows the results, show the filtered
                                // count; otherwise (empty query / everything matches)
                                // show the source total.
                                let found = if filtered_count < total_count {
                                    filtered_count
                                } else {
                                    total_count
                                };
                                format!("Found: {found}")
                            })
                            // Spacer pushes Size and Appearance to the far right of the bar.
                            .child(div().flex_1())
                            .child(
                                div()
                                    .h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child("Size:")
                                    .child(
                                        ButtonGroup::new("size-selector")
                                            .children(Size::ALL.map(|size| {
                                                Button::new(size.label().to_lowercase())
                                                    .label(size.label())
                                                    .selected(selected_size == size)
                                            }))
                                            .on_click(cx.listener(|view, clicks: &Vec<usize>, _, cx| {
                                                if let Some(&index) = clicks.first() {
                                                    view.size = Size::from_index(index);
                                                    cx.notify();
                                                }
                                            })),
                                    ),
                            )
                            .child(
                                div()
                                    .h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child("Appearance:")
                                    .child(
                                        RadioGroup::horizontal("theme")
                                            .children(ThemeModeSelection::ALL.map(|theme| theme.label()))
                                            .selected_index(self.theme.map(|theme| theme.index()))
                                            .on_click(cx.listener(|view, index: &usize, window, cx| {
                                                view.theme = ThemeModeSelection::from_index(*index);
                                                let mode = match view.theme {
                                                    Some(ThemeModeSelection::Light) => ThemeMode::Light,
                                                    Some(ThemeModeSelection::Dark) => ThemeMode::Dark,
                                                    _ => system_theme_mode(window.appearance()),
                                                };
                                                Theme::change(mode, Some(window), cx);
                                            })),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .h_flex()
                            .w_full()
                            .flex_1()
                            .overflow_hidden()
                            .child(
                                div().p_2().border_1().flex_1().h_full().child(
                                    div()
                                        .relative()
                                        .size_full()
                                        .child(
                                            div()
                                                .id("icon-list")
                                                .relative()
                                                .size_full()
                                                .child(list.size_full())
                                                .vertical_scrollbar(&self.scroll_handle),
                                        )
                                        .when(self.searching, |this| {
                                            // Loading overlay: a translucent backdrop with a spinner
                                            // and label, shown while the background filter is running.
                                            this.child(
                                                div()
                                                    .absolute()
                                                    .inset_0()
                                                    .flex()
                                                    .items_center()
                                                    .justify_center()
                                                    .gap_2()
                                                    .bg(cx.theme().background.alpha(0.7))
                                                    .child(
                                                        Spinner::new()
                                                            .small()
                                                            .color(cx.theme().primary),
                                                    )
                                                    .child("Searching..."),
                                            )
                                        }),
                                ),
                            )
                            .child(self.render_info_panel(cx)),
                    )
                    .child({
                        let current = self
                            .selected_icon
                            .as_ref()
                            .map(|e| e.variant_name.as_ref())
                            .unwrap_or("-");
                        let rows = self.visible_rows.borrow();
                        let displayed = if filtered_count == 0 {
                            "0".to_string()
                        } else if rows.is_empty() {
                            "-".to_string()
                        } else {
                            let start = rows.start * columns + 1;
                            let end = (rows.end * columns).min(filtered_count);
                            format!("{start}-{end}")
                        };
                        StatusBar::new().right(format!(
                            "Current: {current} | Icons - displayed: {displayed}, filtered: {filtered_count}, total: {total_count}"
                        ))
                    }),
            )
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
            .into_any_element()
    }
}
