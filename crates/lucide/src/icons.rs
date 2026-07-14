use gpui::{IntoElement, RenderOnce, SharedString};
use gpui_assets_macros::icon_named;
use gpui_component::IconNamed;

// Auto-generated enum of bundled Lucide icons.
//
// Each variant corresponds to an `.svg` file in `assets/icons` and implements
// [`IconNamed`], so it can be used directly with [`gpui_component::Icon`].
//
// ```ignore
// use gpui_component::Icon;
// use gpui_lucide::icons::LucideIcon;
//
// Icon::new(LucideIcon::Check);
// ```
icon_named!(
    LucideIcon,
    "lucide",
    "../../assets/lucide/icons",
    [Debug, Copy, PartialEq, Eq]
);

impl RenderOnce for LucideIcon {
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
            LucideIcon::Check.path().as_ref(),
            format!("{}:/check.svg", crate::LUCIDE_PREFIX)
        );
        assert_eq!(
            LucideIcon::X.path().as_ref(),
            format!("{}:/x.svg", crate::LUCIDE_PREFIX)
        );
        assert_eq!(
            LucideIcon::ArrowRight.path().as_ref(),
            format!("{}:/arrow-right.svg", crate::LUCIDE_PREFIX)
        );
        assert_eq!(
            LucideIcon::Pin.path().as_ref(),
            format!("{}:/pin.svg", crate::LUCIDE_PREFIX)
        );
    }

    #[test]
    fn variants_are_distinct() {
        // Exercises the generated PartialEq (from the derives) and confirms
        // different variants map to different asset paths.
        assert_ne!(LucideIcon::Check, LucideIcon::X);
        assert_ne!(LucideIcon::Check.path(), LucideIcon::X.path());
        assert_eq!(LucideIcon::Check, LucideIcon::Check);
    }
}
