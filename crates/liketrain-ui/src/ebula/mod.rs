use std::{collections::HashMap, time::Duration};

use gpui::{
    Context, Div, Element, FontWeight, InteractiveElement, IntoElement, ParentElement, Pixels,
    Render, ScrollHandle, Size, StatefulInteractiveElement, Styled, Task, Window, div,
    prelude::FluentBuilder, px, size,
};

mod theme;
use gpui_component::{StyledExt, h_flex};
use itertools::Itertools;
use liketrain_core::{
    Direction, Route, SectionEnd, SectionTransition, SwitchId, Track, TrackSectionWaypointType,
    TrainId,
};
pub use theme::*;

use crate::controller::ControllerUiWrapper;

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

#[derive(Debug, Clone)]
enum EbulaEntry {
    Switch(SwitchId),
    Waypoint(TrackSectionWaypointType),
}

impl EbulaEntry {
    pub fn switch_id(&self) -> Option<&SwitchId> {
        match self {
            EbulaEntry::Switch(id) => Some(id),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct EbulaEntries(HashMap<EbulaOffset, Vec<EbulaEntry>>);

impl EbulaEntries {
    pub fn get_slice(&self, offset: &EbulaOffset) -> &[EbulaEntry] {
        self.0.get(offset).map(|v| v.as_slice()).unwrap_or_default()
    }

    pub fn insert(&mut self, offset: EbulaOffset, entry: EbulaEntry) {
        self.0.entry(offset).or_default().push(entry);
    }

    pub fn offsets(&self) -> impl Iterator<Item = EbulaOffset> {
        self.0.keys().copied()
    }

    pub fn offset_range(&self) -> Option<(EbulaOffset, EbulaOffset)> {
        self.offsets().minmax().into_option()
    }
}

/// Emulation of the EBuLa system found in trains of the Deutsche Bahn.
/// For more information see [EBuLa](https://de.wikipedia.org/wiki/EBuLa)
pub struct Ebula {
    theme: EbulaTheme,

    train_id: TrainId,

    /// The precalculated entries. Sorted by offset.
    entries: EbulaEntries,

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
    pub fn new(train_id: TrainId, theme: EbulaTheme, cx: &mut Context<Self>) -> Self {
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

        let controller_state = ControllerUiWrapper::state(cx).read(cx);

        let track = controller_state.track();
        let train = controller_state.train(train_id).unwrap();

        let entries = Self::calculate_entries(track, train.route.as_ref().unwrap());

        Self {
            train_id,
            entries,
            theme,
            content_scroll_handle,
            _task,
        }
    }

    fn calculate_entries(track: &Track, route: &Route) -> EbulaEntries {
        let mut entries = EbulaEntries::default();
        let mut current_meter_offset = 0_f32;

        let mut current_section_direction = route.starting_direction();
        for (section_id, trans) in route.vias_with_transition() {
            let Some(section_geo) = track.section_geo(&section_id) else {
                continue;
            };

            for waypoint in section_geo.waypoints.iter() {
                let waypoint_offset = match current_section_direction {
                    Direction::Forward => waypoint.at_meter,
                    Direction::Backward => section_geo.length - waypoint.at_meter,
                };

                entries.insert(
                    EbulaOffset::from_m(current_meter_offset + waypoint_offset),
                    EbulaEntry::Waypoint(waypoint.r#type.clone()),
                );
            }

            current_meter_offset += section_geo.length;

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

            current_section_direction = match trans.destination_section_end() {
                SectionEnd::Start => Direction::Forward,
                SectionEnd::End => Direction::Backward,
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
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let now = chrono::Local::now();

        let train_name = ControllerUiWrapper::state(cx)
            .read(cx)
            .train(self.train_id)
            .unwrap()
            .data
            .name
            .clone();

        div()
            .w_full()
            .h_10()
            .when(true, |this| self.border_bottom(this))
            .flex()
            .child(self.text_field(BorderSide::Right).w_20().child(train_name))
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
        /*let min_offset = self.entries.keys().copied().min().unwrap_or_default();
        let max_offset = self.entries.keys().copied().max().unwrap_or_default();*/
        let (min_offset, max_offset) = self.entries.offset_range().unwrap_or_default();

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
                                let entries = self.entries.get_slice(&offset);

                                let is_switch = entries.iter().any(|e| e.switch_id().is_some());

                                div()
                                    .w_full()
                                    .h_6()
                                    .flex()
                                    .items_center()
                                    .child(self.m_field(offset.m(), BorderSide::Right).w_24()) // km
                                    .child(div().w_3()) // the line ?
                                    .child(
                                        div()
                                            .h_full()
                                            .flex()
                                            .items_center()
                                            .w_3()
                                            .when(is_switch, |this| {
                                                this.font_weight(FontWeight::BOLD).child("¥")
                                            }),
                                    ) // symbols
                                    .child(
                                        div()
                                            .flex_1()
                                            .when(true, |this| self.border_right(this))
                                            .h_full()
                                            .child(
                                                h_flex()
                                                    .size_full()
                                                    .border_t_2()
                                                    .border_color(self.theme.foreground.alpha(0.2))
                                                    .h_full()
                                                    .gap_2()
                                                    .children(entries.iter().enumerate().map(
                                                        |(idx, e)| {
                                                            let is_last = idx == entries.len() - 1;

                                                            div()
                                                                .when(!is_last, |div| {
                                                                    div.pr_2()
                                                                        .border_r_2()
                                                                        .border_color(
                                                                            self.theme
                                                                                .foreground
                                                                                .alpha(0.2),
                                                                        )
                                                                })
                                                                .child(match e {
                                                                    EbulaEntry::Switch(
                                                                        switch_id,
                                                                    ) => switch_id
                                                                        .to_string()
                                                                        .into_any_element(),
                                                                    EbulaEntry::Waypoint(
                                                                        waypoint_type,
                                                                    ) => div()
                                                                        .child(
                                                                            waypoint_type
                                                                                .to_string(),
                                                                        )
                                                                        .when(
                                                                            waypoint_type
                                                                                .should_highlight(),
                                                                            |div| div.font_bold(),
                                                                        )
                                                                        .into_any_element(),
                                                                })
                                                        },
                                                    )), /*.child(
                                                            self.entries
                                                                .get(&offset)
                                                                .map(|e| match e {
                                                                    EbulaEntry::Switch(switch_id) => {
                                                                        switch_id.to_string()
                                                                    }
                                                                    EbulaEntry::Waypoint(
                                                                        waypoint_type,
                                                                    ) => waypoint_type.to_string(),
                                                                })
                                                                .join(", "),
                                                        ),*/
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
