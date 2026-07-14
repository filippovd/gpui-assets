use gpui::{IntoElement, RenderOnce, SharedString};
use gpui_assets_macros::icon_named;
use gpui_component::IconNamed;

// Auto-generated enum of bundled Material Design Icons.
//
// Each variant corresponds to an `.svg` file in `assets/icons` and implements
// [`IconNamed`], so it can be used directly with [`gpui_component::Icon`].
//
// ```ignore
// use gpui_component::Icon;
// use gpui_mdi::icons::MdiIcon;
//
// Icon::new(MdiIcon::Check);
// ```
icon_named!(
    MdiIcon,
    "mdi",
    "../../assets/mdi/icons",
    [Debug, Copy, PartialEq, Eq]
);

impl RenderOnce for MdiIcon {
    fn render(self, _: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        gpui_component::Icon::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_paths_include_prefix() {
        assert_eq!(
            MdiIcon::Check.path().as_ref(),
            format!("{}:/check.svg", crate::MDI_PREFIX)
        );
        assert_eq!(
            MdiIcon::Close.path().as_ref(),
            format!("{}:/close.svg", crate::MDI_PREFIX)
        );
        assert_eq!(
            MdiIcon::Pin.path().as_ref(),
            format!("{}:/pin.svg", crate::MDI_PREFIX)
        );
        assert_eq!(
            MdiIcon::Star.path().as_ref(),
            format!("{}:/star.svg", crate::MDI_PREFIX)
        );
    }

    #[test]
    fn variants_are_distinct() {
        // Exercises the generated PartialEq (from the derives) and confirms
        // different variants map to different asset paths.
        assert_ne!(MdiIcon::Check, MdiIcon::Close);
        assert_ne!(MdiIcon::Check.path(), MdiIcon::Close.path());
        assert_eq!(MdiIcon::Check, MdiIcon::Check);
    }
}
