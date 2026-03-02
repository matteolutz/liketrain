use std::{
    collections::{HashMap, VecDeque},
    f32::{self},
    rc::Rc,
};

use either::Either;
use gpui::{
    Bounds, IntoElement, PathBuilder, Pixels, Point, RenderOnce, Styled, Window, canvas, point, px,
};
use liketrain_core::{Direction, SectionEnd, SectionId, SectionTransition, SwitchState, Track};
use serde::{Deserialize, Serialize, de::Visitor};
use vek::Vec2;

use crate::layout::{color::LayoutColor, vec::Vec2Ext};

mod color;
mod vec;

const SECTION_STROKE_WIDTH: f32 = 4.0;

const TRANSITION_STROKE_WIDTH: f32 = 4.0;
const TRANSITION_COLOR: LayoutColor = LayoutColor::from_rgb(0, 255, 0);

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayoutSectionId(SectionId);

impl From<SectionId> for LayoutSectionId {
    fn from(id: SectionId) -> Self {
        LayoutSectionId(id)
    }
}

impl From<LayoutSectionId> for SectionId {
    fn from(id: LayoutSectionId) -> Self {
        id.0
    }
}

impl Serialize for LayoutSectionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let formatted = format!("S{}", self.0);
        serializer.serialize_str(&formatted)
    }
}

impl<'de> Deserialize<'de> for LayoutSectionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct LayoutSectionIdVisitor;

        impl<'de> Visitor<'de> for LayoutSectionIdVisitor {
            type Value = LayoutSectionId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string in the format \"S<usize>\"")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if let Some(id_str) = v.strip_prefix("S") {
                    let id = id_str
                        .parse::<usize>()
                        .map_err(|_| E::custom("invalid number in section id"))?;
                    Ok(LayoutSectionId(id.into()))
                } else {
                    Err(E::custom("section id must start with 'S'"))
                }
            }
        }

        deserializer.deserialize_str(LayoutSectionIdVisitor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutSectionGeometry {
    Straight { length: f32 },
    Arc { offset: vek::Vec2<f32>, angle: f32 },
}

#[derive(Clone)]
struct LayoutSectionGeometryRenderResult {
    origin: Point<Pixels>,
    direction: vek::Vec2<f32>,
}

impl LayoutSectionGeometryRenderResult {
    pub fn rotated(&self, angle: f32) -> Self {
        Self {
            origin: self.origin,
            direction: self.direction.rotated_z(angle),
        }
    }

    pub fn opposite(&self) -> Self {
        Self {
            origin: self.origin,
            direction: -self.direction,
        }
    }

    /// Returns a copy of this value with its origin offset in the local space
    /// defined by `direction`.
    ///
    /// The offset is interpreted relative to the direction vector:
    /// - `offset.y` moves the origin **along the direction**
    /// - `offset.x` moves the origin **along the normal (perpendicular)**
    ///   to the direction.
    pub fn with_normal_offset(&self, offset: vek::Vec2<f32>, flip: bool) -> Self {
        let mut angle = self.direction.y.atan2(self.direction.x);
        if flip {
            angle = -angle;
        }

        let local = vek::Vec2::new(offset.y, offset.x);
        let new_origin = vek::Vec2::from_point(self.origin) + local.rotated_z(angle);

        Self {
            direction: self.direction,
            origin: new_origin.to_point(),
        }
    }
}

impl LayoutSectionGeometry {
    fn render(
        &self,
        prev_render_result: LayoutSectionGeometryRenderResult,
        color: LayoutColor,
        reverse: bool,
        window: &mut Window,
    ) -> LayoutSectionGeometryRenderResult {
        let gpui_color: gpui::Rgba = color.into();

        match *self {
            Self::Straight { length } => {
                let mut path_builder = PathBuilder::stroke(px(SECTION_STROKE_WIDTH));
                path_builder.move_to(prev_render_result.origin);

                let destination = prev_render_result.origin
                    + (prev_render_result.direction.to_normalized_point() * length);

                path_builder.line_to(destination);

                let path = path_builder.build().unwrap();
                window.paint_path(path, gpui_color);

                LayoutSectionGeometryRenderResult {
                    origin: destination,
                    direction: prev_render_result.direction,
                }
            }
            Self::Arc { mut offset, angle } => {
                let mut path_builder = PathBuilder::stroke(px(SECTION_STROKE_WIDTH));
                path_builder.move_to(prev_render_result.origin);

                if reverse {
                    offset = -offset;
                }

                let origin = Vec2::from_point(prev_render_result.origin);
                let end =
                    Vec2::from_point(prev_render_result.with_normal_offset(offset, true).origin);

                let prev_direction = prev_render_result.direction;

                let actual_angle = offset.x.atan2(offset.y);

                let angle = angle.abs().to_radians() * actual_angle.signum();

                let new_direction = prev_direction.rotated_z(angle);

                let chord_len = (end - origin).magnitude();
                let abs_angle = angle.abs();

                let r = chord_len / (2.0 * (abs_angle / 2.0).sin());

                path_builder.arc_to(
                    point(px(r), px(r)),
                    px(0.0),
                    false,
                    actual_angle > 0.0,
                    end.to_point(),
                );

                let path = path_builder.build().unwrap();
                window.paint_path(path, gpui_color);

                LayoutSectionGeometryRenderResult {
                    origin: end.to_point(),
                    direction: new_direction,
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSection {
    geometries: Vec<LayoutSectionGeometry>,
    color: LayoutColor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    sections: HashMap<LayoutSectionId, LayoutSection>,
}

struct SectionVisit {
    section_id: LayoutSectionId,

    prev_render_result: LayoutSectionGeometryRenderResult,

    section_direction: Direction,
}

impl Layout {
    fn render_transition(
        &self,
        transition: SectionTransition,
        destination_section_id: LayoutSectionId,
        prev_render_result: &LayoutSectionGeometryRenderResult,
        window: &mut Window,
    ) -> SectionVisit {
        let switch_x_offset = 10.0;
        let switch_y_offset = 10.0;

        let switches = transition.required_switch_changes();
        let normal_offset =
            switches
                .into_iter()
                .fold(vek::Vec2::new(0.0, 0.0), |acc, switch_change| {
                    match (switch_change.is_switch_back, switch_change.required_state) {
                        (false, SwitchState::Left) => {
                            acc + vek::Vec2::new(-switch_x_offset, switch_y_offset)
                        }
                        (false, SwitchState::Right) => {
                            acc + vek::Vec2::new(switch_x_offset, switch_y_offset)
                        }

                        (true, SwitchState::Left) => {
                            acc + vek::Vec2::new(-switch_x_offset, switch_y_offset)
                        }
                        (true, SwitchState::Right) => {
                            acc + vek::Vec2::new(switch_x_offset, switch_y_offset)
                        }
                    }
                });

        let new_render_result = prev_render_result.with_normal_offset(normal_offset, false);

        // draw the transition
        {
            let mut path_builder = PathBuilder::stroke(px(TRANSITION_STROKE_WIDTH));
            path_builder.move_to(prev_render_result.origin);
            path_builder.line_to(new_render_result.origin);
            let path = path_builder.build().unwrap();
            window.paint_path(path, gpui::Rgba::from(TRANSITION_COLOR));
        }

        SectionVisit {
            section_id: destination_section_id,
            prev_render_result: new_render_result,
            section_direction: match transition.destination_section_end() {
                SectionEnd::Start => Direction::Forward,
                SectionEnd::End => Direction::Backward,
            },
        }
    }

    fn render(
        &self,
        track: &Track,
        config: LayoutRenderConfig,
        bounds: Bounds<Pixels>,
        window: &mut Window,
    ) {
        let absolute_starting_point = bounds.origin
            + point(
                config.starting_point.x * bounds.size.width.as_f32(),
                config.starting_point.y * bounds.size.height.as_f32(),
            );
        let mut visited_sections = HashMap::new();

        let mut sections_to_visit = VecDeque::new();

        sections_to_visit.push_back(SectionVisit {
            section_id: config.starting_section.into(),
            prev_render_result: LayoutSectionGeometryRenderResult {
                origin: absolute_starting_point,
                direction: config.forward_direction,
            },
            section_direction: Direction::Forward,
        });

        sections_to_visit.push_back(SectionVisit {
            section_id: config.starting_section.into(),
            prev_render_result: LayoutSectionGeometryRenderResult {
                origin: absolute_starting_point,
                direction: config.forward_direction,
            },
            section_direction: Direction::Backward,
        });

        while let Some(section) = sections_to_visit.pop_front() {
            if visited_sections.contains_key(&section.section_id) {
                continue;
            }

            let Some(layout_section) = self.sections.get(&section.section_id) else {
                continue;
            };

            // draw this section
            let new_render_result = match section.section_direction {
                Direction::Forward => Either::Left(layout_section.geometries.iter()),
                Direction::Backward => Either::Right(layout_section.geometries.iter().rev()),
            }
            .fold(section.prev_render_result.clone(), |render_result, geo| {
                geo.render(
                    render_result,
                    layout_section.color,
                    section.section_direction == Direction::Backward,
                    window,
                )
            });

            visited_sections.insert(
                section.section_id,
                (
                    section.prev_render_result.clone(),
                    new_render_result.clone(),
                ),
            );

            // TODO: get the next sections
            for transition in track
                .transitions(section.section_id.0, section.section_direction)
                // TODO: remove this
                .unwrap_or_default()
            {
                let destination_section_id: LayoutSectionId = transition.destination().into();

                if let Some((connected_render_result, _)) =
                    visited_sections.get(&destination_section_id)
                {
                    let mut path_builder = PathBuilder::stroke(px(SECTION_STROKE_WIDTH));
                    path_builder.move_to(new_render_result.origin);
                    path_builder.line_to(connected_render_result.origin);
                    let path = path_builder.build().unwrap();

                    let gpui_color: gpui::Rgba = layout_section.color.into();
                    window.paint_path(path, gpui_color);

                    continue;
                }

                let visit = self.render_transition(
                    transition,
                    destination_section_id,
                    &new_render_result,
                    window,
                );
                sections_to_visit.push_back(visit);
            }

            /*
            for back_transition in track
                .transitions(section.section_id.0, section.section_direction.opposite())
                .unwrap_or_default()
                .into_iter()
            {
                let destination_section_id = back_transition.destination().into();

                if visited_sections.contains_key(&destination_section_id) {
                    continue;
                }

                let visit = self.render_transition(
                    back_transition,
                    destination_section_id,
                    &section.prev_render_result.opposite(),
                    window,
                );
                sections_to_visit.push_back(visit);
            }*/
        }
    }

    pub fn renderer(&self, track: &Rc<Track>, config: LayoutRenderConfig) -> LayoutRenderer {
        LayoutRenderer {
            layout: self.clone(),
            track: track.clone(),
            config,
        }
    }
}

#[derive(Debug)]
pub struct LayoutRenderConfig {
    /// The section to start rendering from
    pub starting_section: SectionId,

    /// The relative point (0.0..=1.0) from where to start rendering within the starting_section
    pub starting_point: Point<Pixels>,

    /// The angle of the forward direction of the starting_section
    pub forward_direction: vek::Vec2<f32>,
}

#[derive(IntoElement)]
pub struct LayoutRenderer {
    layout: Layout,
    track: Rc<Track>,

    config: LayoutRenderConfig,
}

impl RenderOnce for LayoutRenderer {
    fn render(self, _: &mut gpui::Window, _: &mut gpui::App) -> impl gpui::IntoElement {
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                self.layout.render(&self.track, self.config, bounds, window);
            },
        )
        .size_full()
    }
}
