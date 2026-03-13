use std::rc::Rc;

use gpui::{
    App, Bounds, Context, Entity, SharedString, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};
use liketrain_core::{Track, TrackGeometry, parser::Parser};

use crate::{
    ebula::{Ebula, EbulaTheme},
    layout::{Layout, LayoutRenderer, ResolvedLayout},
};

mod app_ext;
mod assets;
mod ebula;
mod layout;
mod ui;

struct HelloWorld2 {
    text: SharedString,
}

impl Render for HelloWorld2 {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x505050))
            .size(px(500.0))
            .justify_center()
            .items_center()
            .shadow_lg()
            .border_1()
            .border_color(rgb(0x0000ff))
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(format!("Hello, {}!", &self.text))
            .font_family("JetBrains Mono")
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(div().size_8().bg(gpui::red()))
                    .child(div().size_8().bg(gpui::green()))
                    .child(div().size_8().bg(gpui::blue()))
                    .child(div().size_8().bg(gpui::yellow()))
                    .child(div().size_8().bg(gpui::black()))
                    .child(div().size_8().bg(gpui::white())),
            )
    }
}

struct HelloWorld {
    renderer: Entity<LayoutRenderer>,
}

impl HelloWorld {
    pub fn new(track: Rc<Track>, layout: ResolvedLayout, cx: &mut Context<Self>) -> Self {
        Self {
            renderer: cx.new(|cx| layout.renderer(&track, cx)),
        }
    }
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.renderer.clone())
    }
}

fn init_logger() {
    #[cfg(debug_assertions)]
    {
        if std::env::var("RUST_LOG").is_err() {
            unsafe { std::env::set_var("RUST_LOG", "debug") }
        }
    }

    env_logger::init();
}

fn main() {
    init_logger();

    let track_dsl = include_str!("../../../resources/track.ltt");
    let track_geo = include_str!("../../../resources/geo.json");

    let track_defs = liketrain_core::parser::parser().parse(track_dsl).unwrap();
    let track_geo: TrackGeometry = serde_json::from_str(track_geo).unwrap();

    let evaluator = liketrain_core::parser::eval::Evaluator::default();

    let mut track = evaluator.evaluate(track_defs).unwrap();
    track.set_geometry(track_geo);

    let track = Rc::new(track);

    let layout_json = include_str!("../../../resources/layout.json");
    let layout: Layout = serde_json::from_str(layout_json).unwrap();

    let resolved_layout = layout.resolve(&track);

    log::info!("layout: {:#?}", resolved_layout);

    gpui_platform::application()
        .with_assets(assets::Assets)
        .run(|cx: &mut App| {
            assets::init(cx).unwrap();
            log::debug!("fonts: {:#?}", cx.text_system().all_font_names());

            let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| {
                    window.set_window_title("liketrain");
                    cx.new(|cx| HelloWorld::new(track, resolved_layout, cx))
                },
            )
            .unwrap();

            let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| {
                    window.set_window_title("liketrain Test");
                    cx.new(|cx| HelloWorld2 {
                        text: "liketrain".into(),
                    })
                },
            )
            .unwrap();

            let bounds = Bounds::centered(None, Ebula::get_window_size(600.0), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| {
                    window.set_window_title("liketrain - EBuLa");
                    cx.new(|cx| Ebula::new(EbulaTheme::default_light(), cx))
                },
            )
            .unwrap();
        });
}
