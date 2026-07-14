//! A small standalone component that exposes a button-triggered popover
//! explaining the fzf-style search syntax used by the icon gallery.

use gpui::{
    Anchor, Context, InteractiveElement as _, IntoElement, KeyBinding, ParentElement as _, Render,
    Styled, Window, actions, div,
};
use gpui_component::{
    Sizable, StyledExt,
    button::{Button, ButtonVariants},
    description_list::DescriptionList,
    popover::Popover,
};
use gpui_lucide::icons::LucideIcon;

/// Keymap context that scopes Enter/Space handling to the popover trigger, so
/// the bindings don't leak into the search input or other elements.
const CONTEXT: &str = "SearchSyntaxPopover";

actions!(search_syntax, [OpenSearchSyntaxPopover]);

/// A button-triggered popover describing the supported fzf-style search syntax.
/// Anchored above the search input suffix; opens on click or Enter/Space when
/// the trigger is focused.
pub struct SearchSyntaxPopover {
    open: bool,
}

impl SearchSyntaxPopover {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.bind_keys([
            KeyBinding::new("enter", OpenSearchSyntaxPopover, Some(CONTEXT)),
            KeyBinding::new("space", OpenSearchSyntaxPopover, Some(CONTEXT)),
        ]);

        Self { open: false }
    }

    fn toggle(&mut self, cx: &mut Context<Self>) {
        self.open = !self.open;
        cx.notify();
    }

    fn on_open_change(&mut self, open: &bool, cx: &mut Context<Self>) {
        self.open = *open;
        cx.notify();
    }

    /// Render the popover content describing the supported search syntax.
    fn render_help(&self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .gap_3()
            .w(gpui::px(320.0))
            .p_2()
            .child("The search box supports a small fzf-like query language:")
            .child(
                DescriptionList::vertical()
                    .bordered(true)
                    .columns(1)
                    .item("arrow", "fuzzy match 'arrow' anywhere in the name", 1)
                    .item("'circle", "exact substring 'circle'", 1)
                    .item("^arrow", "name must start with 'arrow'", 1)
                    .item("circle$", "name must end with 'circle'", 1)
                    .item("!outline", "exclude names containing 'outline'", 1)
                    .item("arrow !outline", "combine terms with AND / NOT", 1),
            )
            .child("Terms are case-insensitive and whitespace-separated.")
    }
}

impl Render for SearchSyntaxPopover {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        Popover::new("search-syntax-popover")
            .anchor(Anchor::TopCenter)
            .mt_2()
            .open(self.open)
            .on_open_change(cx.listener(|this, open: &bool, _, cx| {
                this.on_open_change(open, cx);
            }))
            .trigger(
                Button::new("search-info")
                    .ghost()
                    .icon(LucideIcon::Info)
                    .xsmall()
                    .tab_stop(true)
                    // Scope Enter/Space to this trigger only (see the `CONTEXT`
                    // bindings in `new`) so the keys don't leak into the search
                    // input or other elements.
                    .key_context(CONTEXT)
                    .tooltip("Search syntax help (fzf-style)")
                    .on_action(
                        cx.listener(|this, _: &OpenSearchSyntaxPopover, _window, cx| {
                            this.toggle(cx);
                        }),
                    ),
            )
            .child(self.render_help(window, cx))
    }
}
