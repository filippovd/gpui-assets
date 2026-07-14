//! The "About" entry in the title bar: a right-aligned button that opens a
//! centered modal `Dialog` describing the application.

use gpui::*;
use gpui_component::{
    ActiveTheme as _, Sizable as _,
    button::{Button, ButtonVariants as _},
    description_list::DescriptionList,
    dialog::{
        Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
    },
    h_flex, v_flex,
};

/// Crate version, injected by Cargo at compile time from the workspace version.
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Render the About button + dialog. The button is meant to be the second
/// child of the title bar's `justify_between` row, so it lands at the right
/// edge (left of the native window controls).
pub(super) fn render_about(cx: &mut App) -> impl gpui::IntoElement {
    Dialog::new(cx)
        .trigger(
            Button::new("about")
                .ghost()
                .label("About")
                .small()
                .tooltip("About this application"),
        )
        .title("GPUI Assets")
        .content(|content, _window, cx| {
            content
                .child(
                    DialogHeader::new().child(
                        v_flex()
                            .gap_1()
                            .child(DialogTitle::new().child("GPUI Assets — Icon Gallery"))
                            .child(
                                DialogDescription::new().child(format!("Version {VERSION}")),
                            ),
                    ),
                )
                .child(
                    v_flex()
                        .px_4()
                        .pb_4()
                        .gap_3()
                        .child(
                            DialogDescription::new().child(
                                "A searchable, filterable gallery of icons from every registered \
                                 asset source: Lucide, MDI, and the gpui-component fallback set.",
                            ),
                        )
                        .child(
                            DescriptionList::vertical()
                                .bordered(true)
                                .columns(1)
                                .item("UI Framework", "Zed gpui", 1)
                                .item("Components", "gpui-component", 1)
                                .item("Icons", "Lucide + Material Design Icons (MDI)", 1),
                        ),
                )
                .child(
                    DialogFooter::new()
                        .px_4()
                        .pb_4()
                        .justify_end()
                        .child(
                            h_flex().gap_2().child(
                                DialogClose::new().child(
                                    Button::new("about-close").primary().label("Close"),
                                ),
                            ),
                        ),
                )
                .bg(cx.theme().background)
        })
}
