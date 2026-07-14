use gpui::*;
use gpui_component::{
    button::{Button, ButtonVariants as _},
    *,
};
use gpui_lucide::{LucideAssets, icons::LucideIcon};
use gpui_mdi::{MdiAssets, icons::MdiIcon};

struct IconDemo;

const ICON_SIZES: [f32; 25] = [
    12.0, 14.0, 16.0, 18.0, 20.0, 22.0, 24.0, 26.0, 28.0, 30.0, 32.0, 34.0, 36.0, 38.0, 40.0, 42.0,
    44.0, 46.0, 48.0, 50.0, 52.0, 54.0, 56.0, 58.0, 60.0,
];

fn icon_grid(icons: Vec<Icon>, sizes: &[f32]) -> impl IntoElement {
    // `cycle()` so icons beyond the fixed size table still get a size instead of
    // being silently dropped by `zip` (which stops at the shorter iterator).
    let cells: Vec<_> = icons
        .into_iter()
        .zip(sizes.iter().copied().cycle())
        .collect();
    div().v_flex().gap_1().children(cells.chunks(5).map(|row| {
        div()
            .h_flex()
            .gap_1()
            .items_center()
            .children(row.iter().map(|(icon, size)| icon.clone().size(px(*size))))
    }))
}

impl Render for IconDemo {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let gpui_icons: Vec<Icon> = vec![
            Icon::new(IconName::ALargeSmall),
            Icon::new(IconName::ArrowDown),
            Icon::new(IconName::ArrowLeft),
            Icon::new(IconName::ArrowRight),
            Icon::new(IconName::ArrowUp),
            Icon::new(IconName::Asterisk),
            Icon::new(IconName::BatteryCharging),
            Icon::new(IconName::BatteryFull),
            Icon::new(IconName::BatteryLow),
            Icon::new(IconName::BatteryMedium),
            Icon::new(IconName::BatteryWarning),
            Icon::new(IconName::Battery),
            Icon::new(IconName::Bell),
            Icon::new(IconName::BookOpen),
            Icon::new(IconName::Bot),
            Icon::new(IconName::Building2),
            Icon::new(IconName::Calendar),
            Icon::new(IconName::CaseSensitive),
            Icon::new(IconName::ChartPie),
            Icon::new(IconName::Check),
            Icon::new(IconName::ChevronDown),
            Icon::new(IconName::ChevronLeft),
            Icon::new(IconName::ChevronRight),
            Icon::new(IconName::ChevronUp),
            Icon::new(IconName::ChevronsUpDown),
        ];

        let lucide_icons: Vec<Icon> = vec![
            Icon::new(LucideIcon::AArrowDown),
            Icon::new(LucideIcon::AArrowUp),
            Icon::new(LucideIcon::ALargeSmall),
            Icon::new(LucideIcon::Accessibility),
            Icon::new(LucideIcon::Activity),
            Icon::new(LucideIcon::Ad),
            Icon::new(LucideIcon::AirVent),
            Icon::new(LucideIcon::Airplay),
            Icon::new(LucideIcon::AlarmClockCheck),
            Icon::new(LucideIcon::AlarmClockMinus),
            Icon::new(LucideIcon::AlarmClockOff),
            Icon::new(LucideIcon::AlarmClockPlus),
            Icon::new(LucideIcon::AlarmClock),
            Icon::new(LucideIcon::AlarmSmoke),
            Icon::new(LucideIcon::Album),
            Icon::new(LucideIcon::AlignCenterHorizontal),
            Icon::new(LucideIcon::AlignCenterVertical),
            Icon::new(LucideIcon::AlignEndHorizontal),
            Icon::new(LucideIcon::AlignEndVertical),
            Icon::new(LucideIcon::AlignHorizontalDistributeCenter),
            Icon::new(LucideIcon::AlignHorizontalDistributeEnd),
            Icon::new(LucideIcon::AlignHorizontalDistributeStart),
            Icon::new(LucideIcon::AlignHorizontalJustifyCenter),
            Icon::new(LucideIcon::AlignHorizontalJustifyEnd),
            Icon::new(LucideIcon::AlignHorizontalJustifyStart),
        ];

        let mdi_icons: Vec<Icon> = vec![
            Icon::new(MdiIcon::AbTesting),
            Icon::new(MdiIcon::Abacus),
            Icon::new(MdiIcon::AbjadArabic),
            Icon::new(MdiIcon::AbjadHebrew),
            Icon::new(MdiIcon::AbugidaDevanagari),
            Icon::new(MdiIcon::AbugidaThai),
            Icon::new(MdiIcon::AccessPointCheck),
            Icon::new(MdiIcon::AccessPointMinus),
            Icon::new(MdiIcon::AccessPointNetworkOff),
            Icon::new(MdiIcon::AccessPointNetwork),
            Icon::new(MdiIcon::AccessPointOff),
            Icon::new(MdiIcon::AccessPointPlus),
            Icon::new(MdiIcon::AccessPointRemove),
            Icon::new(MdiIcon::AccessPoint),
            Icon::new(MdiIcon::AccountAlertOutline),
            Icon::new(MdiIcon::AccountAlert),
            Icon::new(MdiIcon::AccountArrowDownOutline),
            Icon::new(MdiIcon::AccountArrowDown),
            Icon::new(MdiIcon::AccountArrowLeftOutline),
            Icon::new(MdiIcon::AccountArrowLeft),
            Icon::new(MdiIcon::AccountArrowRightOutline),
            Icon::new(MdiIcon::AccountArrowRight),
            Icon::new(MdiIcon::AccountArrowUpOutline),
            Icon::new(MdiIcon::AccountArrowUp),
            Icon::new(MdiIcon::AccountBadgeOutline),
        ];

        div()
            .v_flex()
            .gap_6()
            .size_full()
            .items_center()
            .justify_center()
            .child("Hello, GPUI!")
            .child(
                div()
                    .h_flex()
                    .gap_8()
                    .items_start()
                    .child(
                        div()
                            .v_flex()
                            .gap_3()
                            .items_center()
                            .child("gpui-component")
                            .child(icon_grid(gpui_icons, &ICON_SIZES)),
                    )
                    .child(
                        div()
                            .v_flex()
                            .gap_3()
                            .items_center()
                            .child("lucide")
                            .child(icon_grid(lucide_icons, &ICON_SIZES)),
                    )
                    .child(
                        div()
                            .v_flex()
                            .gap_3()
                            .items_center()
                            .child("mdi")
                            .child(icon_grid(mdi_icons, &ICON_SIZES)),
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
        .use_source(LucideAssets)
        .use_source(MdiAssets)
        .fallback(gpui_component_assets::Assets);
    let app = gpui_platform::application().with_assets(assets);

    app.run(move |cx| {
        // This must be called before using any GPUI Component features.
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                let view = cx.new(|_| IconDemo);
                // The first-level view in every window must be a Root.
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("Failed to open window");
        })
        .detach();
    });
}
