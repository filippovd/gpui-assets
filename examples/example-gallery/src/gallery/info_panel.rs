//! The right-side info panel of the gallery: details of the selected icon
//! plus the Copy/Download actions. Implemented as an `impl IconGallery` block
//! so it can reach the entity's state directly.

use std::fs;

use gpui::*;
use gpui::{ClipboardItem, IntoElement, Window};
use gpui_component::{
    ActiveTheme as _, Icon, IconName, StyledExt, WindowExt as _,
    button::{Button, ButtonVariants},
    description_list::DescriptionList,
    notification::NotificationType,
};
use gpui_lucide::icons::LucideIcon;

use super::IconGallery;
use super::util::{downloads_dir, file_name_from_path, load_svg_content};

/// Width of the right-side info panel. Shared with `gallery.rs` so the icon
/// grid can compute how much horizontal space remains for the list.
pub(super) const INFO_PANEL_WIDTH: Pixels = px(360.0);

impl IconGallery {
    /// Open a Save As dialog for the selected icon and write its SVG to the
    /// chosen path on a background thread, then report success or failure via
    /// a notification. Remembers the destination directory for the next save.
    pub(super) fn download_svg(&mut self, window: &mut Window, cx: &mut gpui::Context<Self>) {
        let Some(entry) = self.selected_icon.as_ref() else {
            return;
        };
        let Some(svg) = load_svg_content(entry) else {
            window.push_notification((NotificationType::Error, "Failed to load SVG"), cx);
            return;
        };

        let file_name = file_name_from_path(entry.path.as_ref()).to_string();
        let directory = self.last_save_dir.clone().unwrap_or_else(downloads_dir);
        let save_dialog = cx.prompt_for_new_path(&directory, Some(&file_name));
        let window_handle = window.window_handle();

        cx.spawn(async move |gallery, cx| {
            // The dialog future nests two results: the outer is the oneshot
            // receiver, the inner is the platform call; `Option<PathBuf>` is
            // `None` when the user cancels.
            let path = match save_dialog.await {
                Ok(Ok(Some(path))) => path,
                Ok(Ok(None)) => return,
                Ok(Err(error)) => {
                    let _ = cx.update_window(window_handle, |_view, window, cx| {
                        window.push_notification(
                            (
                                NotificationType::Error,
                                format!("Failed to choose save location: {error:#}"),
                            ),
                            cx,
                        );
                    });
                    return;
                }
                Err(_) => return,
            };

            // Write off the UI thread so a slow disk doesn't block rendering.
            let saved_path = path.clone();
            let data = svg.into_bytes();
            let result = cx
                .background_spawn(async move { fs::write(&path, data) })
                .await;

            let _ = cx.update_window(window_handle, |_view, window, cx| {
                match result {
                    Ok(()) => {
                        // Remember the destination directory for the next save.
                        gallery
                            .update(cx, |this, _cx| {
                                this.last_save_dir = saved_path.parent().map(|p| p.to_path_buf());
                            })
                            .ok();
                        window.push_notification(
                            (
                                NotificationType::Success,
                                format!("SVG saved to {}", saved_path.display()),
                            ),
                            cx,
                        );
                    }
                    Err(error) => window.push_notification(
                        (
                            NotificationType::Error,
                            format!("Failed to save SVG: {error:#}"),
                        ),
                        cx,
                    ),
                }
            });
        })
        .detach();
    }

    /// Render the right-side panel showing details of the selected icon.
    pub(super) fn render_info_panel(&self, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let Some(entry) = &self.selected_icon else {
            return div()
                .w(INFO_PANEL_WIDTH)
                .h_full()
                .flex()
                .items_center()
                .justify_center()
                .border_l_1()
                .border_color(cx.theme().border)
                .p_4()
                .child(
                    div()
                        .text_color(cx.theme().muted_foreground)
                        .child("Select an icon to view details"),
                )
                .into_any_element();
        };

        let preview_size = gpui::px(256.0);
        let source_label = entry.source.label();
        let path = entry.path.to_string();
        let file_name = file_name_from_path(&path).to_string();
        let variant = entry.variant_name.to_string();

        div()
            .w(INFO_PANEL_WIDTH)
            .h_full()
            .border_l_1()
            .border_color(cx.theme().border)
            .p_4()
            .v_flex()
            .gap_4()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(Icon::empty().path(entry.path.clone()).size(preview_size)),
            )
            .child(
                DescriptionList::vertical()
                    .bordered(true)
                    .columns(1)
                    .item(
                        div()
                            .h_flex()
                            .items_center()
                            .justify_between()
                            .gap_2()
                            .child("Variant")
                            .child(
                                Button::new("copy-variant")
                                    .icon(Icon::new(IconName::Copy))
                                    .ghost()
                                    .compact()
                                    .tooltip("Copy variant name")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        if let Some(entry) = &this.selected_icon {
                                            cx.write_to_clipboard(ClipboardItem::new_string(
                                                entry.variant_name.to_string(),
                                            ));
                                            window.push_notification(
                                                (
                                                    NotificationType::Success,
                                                    "Variant name copied to clipboard",
                                                ),
                                                cx,
                                            );
                                        }
                                    })),
                            )
                            .into_any_element(),
                        variant,
                        1,
                    )
                    .item("Source", source_label, 1)
                    .item(
                        div()
                            .h_flex()
                            .items_center()
                            .justify_between()
                            .gap_2()
                            .child("Asset Path")
                            .child(
                                Button::new("copy-asset-path")
                                    .icon(Icon::new(IconName::Copy))
                                    .ghost()
                                    .compact()
                                    .tooltip("Copy asset path")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        if let Some(entry) = &this.selected_icon {
                                            cx.write_to_clipboard(ClipboardItem::new_string(
                                                entry.path.to_string(),
                                            ));
                                            window.push_notification(
                                                (
                                                    NotificationType::Success,
                                                    "Asset path copied to clipboard",
                                                ),
                                                cx,
                                            );
                                        }
                                    })),
                            )
                            .into_any_element(),
                        path,
                        1,
                    )
                    .item(
                        div()
                            .h_flex()
                            .items_center()
                            .justify_between()
                            .gap_2()
                            .child("File Name")
                            .child(
                                div()
                                    .h_flex()
                                    .gap_1()
                                    .child(
                                        Button::new("copy-svg")
                                            .icon(Icon::new(IconName::Copy))
                                            .ghost()
                                            .compact()
                                            .tooltip("Copy SVG")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                if let Some(entry) = &this.selected_icon {
                                                    match load_svg_content(entry) {
                                                        Some(svg) => {
                                                            cx.write_to_clipboard(
                                                                ClipboardItem::new_string(svg),
                                                            );
                                                            window.push_notification(
                                                                (
                                                                    NotificationType::Success,
                                                                    "SVG copied to clipboard",
                                                                ),
                                                                cx,
                                                            );
                                                        }
                                                        None => {
                                                            window.push_notification(
                                                                (
                                                                    NotificationType::Error,
                                                                    "Failed to load SVG",
                                                                ),
                                                                cx,
                                                            );
                                                        }
                                                    }
                                                }
                                            })),
                                    )
                                    .child(
                                        Button::new("download-svg")
                                            .icon(Icon::new(LucideIcon::Download))
                                            .ghost()
                                            .compact()
                                            .tooltip("Download SVG")
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.download_svg(window, cx);
                                            })),
                                    ),
                            )
                            .into_any_element(),
                        file_name,
                        1,
                    ),
            )
            .into_any_element()
    }
}
