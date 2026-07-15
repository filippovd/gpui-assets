use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants as _},
    *,
};

mod assets;
mod icons;

use assets::{CustomLucideAssets, CustomMdiAssets};
use icons::{CustomLucideIcon, CustomMdiIcon};

struct CustomPrefixDemo;

impl Render for CustomPrefixDemo {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .v_flex()
            .gap_6()
            .size_full()
            .items_center()
            .justify_center()
            .child("Custom Prefix & Custom Sources Demo")
            .child(
                div()
                    .h_flex()
                    .gap_8()
                    .items_center()
                    .child(
                        div()
                            .v_flex()
                            .gap_3()
                            .items_center()
                            .child(assets::CUSTOM_LUCIDE_PREFIX)
                            .child(Icon::new(CustomLucideIcon::ALargeSmall).size(px(32.0)))
                            .child(Icon::new(CustomLucideIcon::Accessibility).size(px(32.0)))
                            .child(Icon::new(CustomLucideIcon::Activity).size(px(32.0))),
                    )
                    .child(
                        div()
                            .v_flex()
                            .gap_3()
                            .items_center()
                            .child(assets::CUSTOM_MDI_PREFIX)
                            .child(Icon::new(CustomMdiIcon::AbTesting).size(px(32.0)))
                            .child(Icon::new(CustomMdiIcon::Abacus).size(px(32.0)))
                            .child(Icon::new(CustomMdiIcon::AccessPoint).size(px(32.0))),
                    ),
            )
            .child(
                Button::new("ok")
                    .primary()
                    .label("Let's Go!")
                    .on_click(|_, _, _| println!("Clicked!")),
            )
    }
}

fn main() {
    let assets = gpui_assets::AssetsRegistry::new()
        .use_source(CustomLucideAssets)
        .use_source(CustomMdiAssets)
        .fallback(gpui_component_assets::Assets);
    let app = gpui_platform::application().with_assets(assets);

    app.run(move |cx| {
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|_| CustomPrefixDemo);
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("Failed to open window");
        })
        .detach();
    });
}
