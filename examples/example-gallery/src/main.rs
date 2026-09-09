//! Icon gallery example: a searchable, filterable grid of icons from every
//! registered asset source (Lucide, MDI, and the gpui-kit fallback).

mod gallery;
mod icons;
mod search_syntax_popover;

use gpui_kit::*;
use gpui_kit::component::{Root, TitleBar};
use gpui_lucide::LucideAssets;
use gpui_mdi::MdiAssets;

use gallery::IconGallery;

fn main() {
    let assets = gpui_assets::AssetsRegistry::new()
        .use_source(LucideAssets)
        .use_source(MdiAssets)
        .fallback(gpui_kit::assets::Assets);
    let app = gpui_kit::application().with_assets(assets);

    app.run(move |cx| {
        gpui_kit::init(cx);

        let window_bounds = Some(WindowBounds::centered(size(px(1185.0), px(780.0)), cx));

        cx.spawn(async move |cx| {
            let window_options = WindowOptions {
                titlebar: Some(TitleBar::title_bar_options()),
                window_bounds,
                ..Default::default()
            };
            cx.open_window(window_options, |window, cx| {
                let view = cx.new(|cx| IconGallery::new(window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("Failed to open window");
        })
        .detach();
    });
}
