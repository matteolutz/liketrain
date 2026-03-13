use std::time::Duration;

use gpui::{
    Context, Div, FontWeight, InteractiveElement, IntoElement, ParentElement, Pixels, Render,
    ScrollHandle, Size, StatefulInteractiveElement, Styled, Task, Window, div,
    prelude::FluentBuilder, px, size,
};

mod theme;
pub use theme::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum BorderSide {
    None,

    Top,
    Right,
    Bottom,
    Left,
}

/// Emulation of the EBuLa system found in trains of the Deutsche Bahn.
/// For more information see [EBuLa](https://de.wikipedia.org/wiki/EBuLa)
pub struct Ebula {
    theme: EbulaTheme,

    content_scroll_handle: ScrollHandle,

    _task: Task<()>,
}

impl Ebula {
    pub fn get_window_size(width: impl Into<Pixels>) -> Size<Pixels> {
        let width = width.into();
        size(width, width * 0.72)
    }
}

impl Ebula {
    pub fn new(theme: EbulaTheme, cx: &mut Context<Self>) -> Self {
        let _task = cx.spawn(async |this, cx| {
            loop {
                let now = chrono::Local::now();
                let missing_to_second = 1000 - now.timestamp_subsec_millis();

                cx.background_executor()
                    .timer(Duration::from_millis(missing_to_second as u64))
                    .await;

                let _ = this.update(cx, |_, cx| cx.notify());
            }
        });

        let content_scroll_handle = ScrollHandle::default();
        content_scroll_handle.scroll_to_bottom();

        Self {
            theme,
            content_scroll_handle,
            _task,
        }
    }
}

impl Ebula {
    fn border<E>(&self, element: E, border_side: BorderSide) -> E
    where
        E: Styled,
    {
        match border_side {
            BorderSide::None => return element,
            BorderSide::Top => element.border_t(self.theme.border_width),
            BorderSide::Right => element.border_r(self.theme.border_width),
            BorderSide::Bottom => element.border_b(self.theme.border_width),
            BorderSide::Left => element.border_l(self.theme.border_width),
        }
        .border_color(self.theme.foreground)
    }

    fn border_bottom<E>(&self, element: E) -> E
    where
        E: Styled,
    {
        self.border(element, BorderSide::Bottom)
    }

    fn border_top<E>(&self, element: E) -> E
    where
        E: Styled,
    {
        self.border(element, BorderSide::Top)
    }

    fn border_right<E>(&self, element: E) -> E
    where
        E: Styled,
    {
        self.border(element, BorderSide::Right)
    }

    fn text_field(&self, border: BorderSide) -> Div {
        div()
            .px_2()
            .h_full()
            .when(true, |this| self.border(this, border))
            .flex()
            .justify_center()
            .items_center()
            .font_weight(FontWeight::BOLD)
    }

    fn km_field(&self, km: f32, border: BorderSide) -> Div {
        let km_int = km.trunc() as u32;
        let km_frac = (km.fract() * 10.0) as u32;

        div()
            .px_2()
            .h_full()
            .when(true, |this| self.border(this, border))
            .flex()
            .justify_center()
            .items_end()
            .font_weight(FontWeight::SEMIBOLD)
            .child(div().text_sm().child(format!("{}", km_int)))
            .child(div().text_xs().pb(px(1.0)).child(format!(",{}", km_frac)))
    }

    fn time_field(&self, hour: u32, minute: u32, second: u32, border: BorderSide) -> Div {
        div()
            .h_full()
            .when(true, |this| self.border(this, border))
            .flex()
            .justify_center()
            .items_end()
            .font_weight(FontWeight::BOLD)
            .child(div().text_sm().child(format!("{:02}:{:02}", hour, minute)))
            .child(div().text_xs().pb(px(1.0)).child(format!(".{}", second)))
    }

    pub fn render_header(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let now = chrono::Local::now();

        div()
            .w_full()
            .h_10()
            .when(true, |this| self.border_bottom(this))
            .flex()
            .child(self.text_field(BorderSide::Right).w_20().child("10901"))
            .child(
                self.text_field(BorderSide::Right)
                    .flex_1()
                    .child("Fahrplan gültig!"),
            )
            .child(
                self.text_field(BorderSide::Right)
                    .w_24()
                    .child(now.format("%d.%m.%Y").to_string()),
            )
            .child(
                self.text_field(BorderSide::None)
                    .border_r_0() // no border for the last element
                    .w_24()
                    .child(now.format("%H:%M:%S").to_string()),
            )
    }

    pub fn render_upcoming(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .w_full()
            .h_5()
            .when(true, |this| self.border_bottom(this))
            .flex()
            .text_sm()
            .child(
                self.text_field(BorderSide::Right)
                    .child("ab km 58,4: 80km/h"),
            )
            .child(self.text_field(BorderSide::Right).flex_1())
            .child(
                self.text_field(BorderSide::None)
                    .border_r_0() // no border for the last element
                    .child("Nächster Halt: KMG HBf"),
            )
    }

    pub fn render_content(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .w_full()
            .flex_1()
            .overflow_y_hidden()
            .when(true, |this| self.border_bottom(this))
            .flex()
            .child(
                div()
                    .h_full()
                    .w_32()
                    .when(true, |this| self.border_right(this)),
            )
            .child(
                div()
                    .h_full()
                    .flex_1()
                    .id("ebula-content")
                    .overflow_y_scroll()
                    .track_scroll(&self.content_scroll_handle)
                    .child(
                        div()
                            .flex()
                            .flex_col_reverse()
                            // .overflow_y_scrollbar()
                            .children((50..=100).map(|i| {
                                let km = i as f32 / 5.0; // 0.2, 0.4, ..

                                div()
                                    .w_full()
                                    .h_6()
                                    .flex()
                                    .items_center()
                                    .child(self.km_field(km, BorderSide::Right).w_24()) // km
                                    .child(div().w_3()) // the line ?
                                    .child(div().w_3()) // symbols
                                    .child(
                                        div()
                                            .flex_1()
                                            .when(true, |this| self.border_right(this))
                                            .child(
                                                div()
                                                    .size_full()
                                                    .border_t_2()
                                                    .border_color(self.theme.foreground.alpha(0.2))
                                                    .child("Test"),
                                            )
                                            .px_2(),
                                    ) // name
                                    .child(self.time_field(1, 1, 1, BorderSide::Right).w_20()) // time of arrival
                                    .child(self.time_field(1, 1, 1, BorderSide::None).w_20()) // time of departure
                            }))
                            .child(
                                div()
                                    .w_full()
                                    .when(true, |this| self.border_bottom(this))
                                    .text_sm()
                                    .px_2()
                                    .text_color(self.theme.foreground.alpha(0.75))
                                    .child("Ende des Fahrplans"),
                            ),
                    ),
            )
    }

    pub fn render_footer(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div().w_full().h_7()
    }
}

impl Render for Ebula {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(self.theme.background)
            .text_color(self.theme.foreground)
            .overflow_y_hidden()
            .child(self.render_header(window, cx))
            .child(self.render_upcoming(window, cx))
            .child(self.render_content(window, cx))
            .child(self.render_footer(window, cx))
    }
}
