use std::fs::File;

use gpui::{
    App, Bounds, Context, Window, WindowBounds, WindowOptions, canvas, fill, prelude::*, px, size,
};

struct HelloWorld {}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        canvas(
            |_, _, _| {},
            |bounds, _, window, _| {
                window.paint_quad(fill(bounds, gpui::red()));
            },
        )
        .size_full()
    }
}

fn main() {
    let xml_file = File::open("C:\\Users\\Matteo\\Downloads\\train track.svg").unwrap();
    let _xml_doc = kiss_xml::parse_stream(xml_file).unwrap();

    gpui_platform::application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| HelloWorld {}),
        )
        .unwrap();
    });
}
