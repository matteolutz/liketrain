use std::rc::Rc;

use gpui::{
    App, Bounds, Context, MouseDownEvent, MouseMoveEvent, Pixels, Point, ScrollWheelEvent, Window,
    WindowBounds, WindowOptions, div, point, prelude::*, px, size,
};
use liketrain_core::{Track, parser::Parser};

use crate::layout::{Layout, LayoutRenderConfig, ResolvedLayout};

mod layout;

struct HelloWorld {
    layout: ResolvedLayout,
    track: Rc<Track>,

    zoom: f32,
    center: Point<Pixels>,

    last_mouse_pos: Option<Point<Pixels>>,
}

impl HelloWorld {
    pub fn new(track: Rc<Track>, layout: ResolvedLayout, _cx: &mut Context<Self>) -> Self {
        let zoom = 1.0;
        let center = point(px(0.0), px(0.0));

        Self {
            layout,
            track,

            zoom,
            center,

            last_mouse_pos: None,
        }
    }
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .on_scroll_wheel(cx.listener(|this, evt: &ScrollWheelEvent, _, cx| {
                let delta_y = evt.delta.pixel_delta(px(1.0)).y.as_f32();
                this.zoom = (this.zoom + delta_y).max(0.0);
                cx.notify();
            }))
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, evt: &MouseDownEvent, _, cx| {
                    this.last_mouse_pos = Some(evt.position);
                    cx.notify();
                }),
            )
            .on_mouse_up(
                gpui::MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.last_mouse_pos = None;
                    cx.notify();
                }),
            )
            .on_mouse_move(cx.listener(|this, evt: &MouseMoveEvent, _, cx| {
                let Some(last_mouse_pos) = this.last_mouse_pos.as_mut() else {
                    return;
                };

                let delta = evt.position - *last_mouse_pos;
                *last_mouse_pos = evt.position;

                this.center -= delta / this.zoom;

                cx.notify();
            }))
            .size_full()
            .child(self.layout.renderer(
                &self.track,
                LayoutRenderConfig {
                    zoom: self.zoom,
                    center: self.center,
                },
            ))
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

    let resolved_layout = layout.resolve(&track);

    log::info!("layout: {:#?}", resolved_layout);

    gpui_platform::application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|cx| HelloWorld::new(track, resolved_layout, cx)),
        )
        .unwrap();
    });
}
