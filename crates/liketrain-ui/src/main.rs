use std::rc::Rc;

use gpui::{
    App, Bounds, Context, Entity, Window, WindowBounds, WindowOptions, div, prelude::*, px, size,
};
use liketrain_core::{Direction, Route, Track, TrackGeometry, Train, parser::Parser};

use crate::{
    ebula::{Ebula, EbulaTheme},
    layout::{Layout, LayoutRenderer, ResolvedLayout},
};

mod app_ext;
mod assets;
mod ebula;
mod layout;
mod ui;

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

    let test_route = Route::new([24_usize, 22, 21, 24], Direction::Forward, &track).unwrap();
    let test_train = Train::from_route("RE5", test_route);

    gpui_platform::application()
        .with_assets(assets::Assets)
        .run(move |cx: &mut App| {
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
                    cx.new(|cx| HelloWorld::new(track.clone(), resolved_layout, cx))
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
                    cx.new(|cx| {
                        Ebula::new(track.clone(), test_train, EbulaTheme::default_light(), cx)
                    })
                },
            )
            .unwrap();
        });
}
