use std::{collections::HashMap, rc::Rc, time::Duration};

use gpui::{
    Context, Div, FontWeight, InteractiveElement, IntoElement, ParentElement, Pixels, Render,
    ScrollHandle, Size, StatefulInteractiveElement, Styled, Task, Window, div,
    prelude::FluentBuilder, px, size,
};

mod theme;
use liketrain_core::{SectionTransition, SwitchId, Track, Train, TrainDrivingMode};
pub use theme::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum BorderSide {
    None,

    Top,
    Right,
    Bottom,
    Left,
}

/// The offset in the EBuLa system.
/// 1 means 25m, 2 means 50m, 3 means 75m, etc.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
struct EbulaOffset(u32);

impl EbulaOffset {
    pub fn from_km(km: f32) -> Self {
        EbulaOffset((km / 0.025) as u32)
    }

    pub fn from_m(m: f32) -> Self {
        EbulaOffset((m / 25.0) as u32)
    }

    pub fn km(&self) -> f32 {
        self.0 as f32 * 0.025
    }

    pub fn m(&self) -> f32 {
        self.0 as f32 * 25.0
    }

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EbulaEntry {
    Switch(SwitchId),
    Waypoint,
}

impl EbulaEntry {
    pub fn switch_id(&self) -> Option<&SwitchId> {
        match self {
            EbulaEntry::Switch(id) => Some(id),
            EbulaEntry::Waypoint => None,
        }
    }
}

/// Emulation of the EBuLa system found in trains of the Deutsche Bahn.
/// For more information see [EBuLa](https://de.wikipedia.org/wiki/EBuLa)
pub struct Ebula {
    theme: EbulaTheme,

    track: Rc<Track>,
    train: Train,

    /// The precalculated entries. Sorted by offset.
    entries: HashMap<EbulaOffset, EbulaEntry>,

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
    pub fn new(track: Rc<Track>, train: Train, theme: EbulaTheme, cx: &mut Context<Self>) -> Self {
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

        let entries = Self::calculate_entries(&track, &train);

        Self {
            track,
            train,
            entries,
            theme,
            content_scroll_handle,
            _task,
        }
    }

    fn calculate_entries(track: &Track, train: &Train) -> HashMap<EbulaOffset, EbulaEntry> {
        let TrainDrivingMode::Route { route, .. } = train.driving_mode() else {
            return HashMap::new();
        };

        let mut entries = HashMap::new();
        let mut current_meter_offset = 0_f32;

        for (section_id, trans) in route.vias_with_transition() {
            let Some(section_geo) = track.section_geo(&section_id) else {
                continue;
            };

            for _waypoint in section_geo.waypoints.iter() {}

            current_meter_offset += section_geo.length;
            log::debug!("current_meter_offset = {}", current_meter_offset);

            match trans {
                SectionTransition::Switch { switch_id, .. }
                | SectionTransition::SwitchBack { switch_id, .. } => {
                    entries.insert(
                        EbulaOffset::from_m(current_meter_offset),
                        EbulaEntry::Switch(switch_id.clone()),
                    );
                }
                _ => {}
            }
        }

        entries
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

    fn m_field(&self, m: f32, border: BorderSide) -> Div {
        let m_int = m.trunc() as u32;
        let m_frac = (m.fract() * 100.0) as u32;

        div()
            .px_2()
            .h_full()
            .when(true, |this| self.border(this, border))
            .flex()
            .justify_center()
            .items_end()
            .font_weight(FontWeight::SEMIBOLD)
            .child(div().text_sm().child(format!("{}", m_int)))
            .child(div().text_xs().pb(px(1.0)).child(format!(",{:02}", m_frac)))
    }

    fn time_field(&self, time: Option<(u32, u32, u32)>, border: BorderSide) -> Div {
        div()
            .h_full()
            .when(true, |this| self.border(this, border))
            .flex()
            .justify_center()
            .items_end()
            .font_weight(FontWeight::BOLD)
            .when_some(time, |this, (hour, minute, second)| {
                this.child(div().text_sm().child(format!("{:02}:{:02}", hour, minute)))
                    .child(div().text_xs().pb(px(1.0)).child(format!(".{}", second)))
            })
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
            .child(
                self.text_field(BorderSide::Right)
                    .w_20()
                    .child(self.train.name().to_string()),
            )
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
        let min_offset = self.entries.keys().copied().min().unwrap_or_default();
        let max_offset = self.entries.keys().copied().max().unwrap_or_default();

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
                            .children((min_offset.0..=max_offset.0).map(|offset| {
                                let offset = EbulaOffset(offset);
                                let entry = self.entries.get(&offset);

                                div()
                                    .w_full()
                                    .h_6()
                                    .flex()
                                    .items_center()
                                    .child(self.m_field(offset.m(), BorderSide::Right).w_24()) // km
                                    .child(div().w_3()) // the line ?
                                    .child(
                                        div().h_full().flex().items_center().w_3().when_some(
                                            entry.and_then(|e| e.switch_id()),
                                            |this, _| this.font_weight(FontWeight::BOLD).child("¥"),
                                        ),
                                    ) // symbols
                                    .child(
                                        div()
                                            .flex_1()
                                            .when(true, |this| self.border_right(this))
                                            .h_full()
                                            .child(
                                                div()
                                                    .size_full()
                                                    .border_t_2()
                                                    .border_color(self.theme.foreground.alpha(0.2))
                                                    .h_full()
                                                    .when_some(entry, |this, entry| {
                                                        this.child(match entry {
                                                            EbulaEntry::Switch(switch_id) => {
                                                                switch_id.to_string()
                                                            }
                                                            EbulaEntry::Waypoint => {
                                                                "Waypoint".to_string()
                                                            }
                                                        })
                                                    }),
                                            )
                                            .px_2(),
                                    ) // name
                                    .child(self.time_field(None, BorderSide::Right).w_20()) // time of arrival
                                    .child(self.time_field(None, BorderSide::None).w_20()) // time of departure
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
            .font_family("Arial")
            .overflow_y_hidden()
            .child(self.render_header(window, cx))
            .child(self.render_upcoming(window, cx))
            .child(self.render_content(window, cx))
            .child(self.render_footer(window, cx))
    }
}
