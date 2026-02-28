use std::rc::Rc;

use gpui::{
    App, Bounds, Context, Window, WindowBounds, WindowOptions, point, prelude::*, px, size,
};
use liketrain_core::{Track, parser::Parser};

use crate::layout::{Layout, LayoutRenderConfig};

mod layout;

struct HelloWorld {
    layout: Layout,
    track: Rc<Track>,
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.layout.renderer(
            &self.track,
            LayoutRenderConfig {
                starting_section: 23_usize.into(),
                starting_point: point(px(0.5), px(0.7)),
                forward_direction: vek::Vec2 { x: 0.0, y: -1.0 },
            },
        )
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

    let track_defs = liketrain_core::parser::parser().parse(track_dsl).unwrap();

    let evaluator = liketrain_core::parser::eval::Evaluator::default();
    let track = evaluator.evaluate(track_defs).unwrap();
    let track = Rc::new(track);

    let layout_json = include_str!("../../../resources/layout.json");
    let layout: Layout = serde_json::from_str(layout_json).unwrap();

    log::info!("layout: {:?}", layout);

    gpui_platform::application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| HelloWorld { track, layout }),
        )
        .unwrap();
    });
}
