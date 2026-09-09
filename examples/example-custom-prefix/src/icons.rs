use gpui_kit::{App, IntoElement, RenderOnce, SharedString, Window};
use gpui_assets_macros::icon_named;
use gpui_kit::component::{Icon, IconNamed};

icon_named!(
    CustomLucideIcon,
    "custom-lucide",
    "assets/lucide/icons",
    [Debug, Copy, PartialEq, Eq]
);

impl RenderOnce for CustomLucideIcon {
    fn render(self, _: &mut Window, _cx: &mut App) -> impl IntoElement {
        Icon::new(self)
    }
}

icon_named!(
    CustomMdiIcon,
    "custom-mdi",
    "assets/mdi/icons",
    [Debug, Copy, PartialEq, Eq]
);

impl RenderOnce for CustomMdiIcon {
    fn render(self, _: &mut Window, _cx: &mut App) -> impl IntoElement {
        Icon::new(self)
    }
}
