use std::{
    collections::{HashMap, HashSet, VecDeque},
    f32,
    rc::Rc,
};

use gpui::{
    AbsoluteLength, Bounds, IntoElement, PathBuilder, Pixels, Point, RenderOnce, Styled, Window,
    canvas, http_client::http::header::SEC_WEBSOCKET_ACCEPT, point, px,
};
use liketrain_core::{Direction, SectionEnd, SectionId, SwitchState, Track};
use serde::{Deserialize, Serialize, de::Visitor};
use vek::{Mat2, Vec2};

use crate::layout::vec::Vec2Ext;

mod vec;

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
    Straight {
        length: f32,
    },
    Arc {
        angle: f32,
        distance: f32,
        tightness: f32,
    },
}

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
}

impl LayoutSectionGeometry {
    fn render(
        &self,
        prev_render_result: LayoutSectionGeometryRenderResult,
        window: &mut Window,
    ) -> LayoutSectionGeometryRenderResult {
        match self {
            &Self::Straight { length } => {
                let mut path_builder = PathBuilder::stroke(px(4.0));
                path_builder.move_to(prev_render_result.origin);

                let destination = prev_render_result.origin
                    + (prev_render_result.direction.to_normalized_point() * length);

                path_builder.line_to(destination);

                let path = path_builder.build().unwrap();
                window.paint_path(path, gpui::red());

                LayoutSectionGeometryRenderResult {
                    origin: destination,
                    direction: prev_render_result.direction,
                }
            }
            &Self::Arc {
                tightness,
                angle,
                distance,
            } => {
                let mut path_builder = PathBuilder::stroke(px(4.0));
                path_builder.move_to(prev_render_result.origin);

                let origin = Vec2::from_point(prev_render_result.origin);
                let new_direction = prev_render_result.direction.rotated_z(angle.to_radians());

                let end = origin + (new_direction * distance);

                // The chord vector from p1 to p2
                let chord = Vec2::new(end.x - origin.x, end.y - origin.y);
                let chord_len = chord.magnitude();

                // The center of the arc must lie on the line perpendicular to `dir` at `p1`.
                // Let's call the perpendicular direction `perp`. The sign of `angle`
                // tells us which side to put the center on.
                //
                // For the arc to be tangent to `dir` at `p1`, the vector from `p1` to
                // the center must be perpendicular to `dir`.
                //
                // If angle > 0 (turning left in screen coords), center is to the left of dir.
                // If angle < 0 (turning right), center is to the right.

                // Perpendicular to dir (rotated -90°, i.e., to the right of travel)
                let perp_right = Vec2::new(
                    prev_render_result.direction.y,
                    -prev_render_result.direction.x,
                );

                // The center is at p1 + r * perp, where perp points toward the center.
                // We need to find r such that |center - p2| = r as well (both points on circle).
                //
                // center = p1 + r * perp
                // |center - p2|² = r²
                //
                // |(p1 + r*perp) - p2|² = r²
                // |r*perp - chord|² = r²
                // r²|perp|² - 2r(perp · chord) + chord² = r²
                // Since |perp| = 1:
                // r² - 2r(perp · chord) + chord² = r²
                // -2r(perp · chord) + chord² = 0
                // r = chord² / (2 * perp · chord)
                //
                // The sign of r tells us which perpendicular direction is correct.

                let dot = perp_right.x * chord.x + perp_right.y * chord.y;

                // r_min is the tightest possible radius (smallest circle tangent to dir at p1
                // that passes through p2)
                let r_min = (chord_len * chord_len) / (2.0 * dot.abs());

                // Apply tightness: scale up the radius for gentler arcs
                let r = r_min * tightness;

                // Determine sweep direction based on which side the turn goes.
                // dot > 0 means p2 is to the right of dir → center is to the right → clockwise sweep
                // dot < 0 means p2 is to the left → center is to the left → counter-clockwise sweep
                let sweep = dot < 0.0;

                // For tightness > 1.0, the arc center moves further out, which means
                // we might need large_arc. Check by computing the arc's subtended angle.
                //
                // With the center known, compute the angle from center→p1 to center→p2.
                let perp_toward_center = if dot > 0.0 {
                    perp_right
                } else {
                    Vec2::new(-perp_right.x, -perp_right.y)
                };
                let center = Vec2::new(
                    origin.x + r * perp_toward_center.x,
                    origin.y + r * perp_toward_center.y,
                );

                let cp1 = Point::new(origin.x - center.x, origin.y - center.y);
                let cp2 = Point::new(end.x - center.x, end.y - center.y);

                let cross = cp1.x * cp2.y - cp1.y * cp2.x;
                let dot_cp = cp1.x * cp2.x + cp1.y * cp2.y;
                let subtended = cross.atan2(dot_cp).abs();

                let large_arc = subtended > f32::consts::PI;

                path_builder.arc_to(
                    point(px(r), px(r)),
                    px(0.0),
                    large_arc,
                    sweep,
                    end.to_point(),
                );

                let path = path_builder.build().unwrap();
                window.paint_path(path, gpui::red());

                LayoutSectionGeometryRenderResult {
                    origin: end.to_point(),
                    direction: new_direction,
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    sections: HashMap<LayoutSectionId, Vec<LayoutSectionGeometry>>,
}

impl Layout {
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

        struct SectionVisit {
            section_id: LayoutSectionId,

            render_result: LayoutSectionGeometryRenderResult,

            section_direction: Direction,
        }

        let mut visited_sections = HashSet::new();

        let mut sections_to_visit = VecDeque::new();

        sections_to_visit.push_back(SectionVisit {
            section_id: config.starting_section.into(),
            render_result: LayoutSectionGeometryRenderResult {
                origin: absolute_starting_point,
                direction: config.forward_direction,
            },
            section_direction: Direction::Forward,
        });

        while let Some(section) = sections_to_visit.pop_front() {
            if visited_sections.contains(&section.section_id) {
                continue;
            }

            let Some(section_geometry) = self.sections.get(&section.section_id) else {
                continue;
            };

            // draw this section
            let new_render_result = section_geometry
                .iter()
                .fold(section.render_result, |render_result, geo| {
                    geo.render(render_result, window)
                });

            visited_sections.insert(section.section_id);

            // TODO: get the next sections
            for transition in track
                .transitions(section.section_id.0, section.section_direction)
                // TODO: remove this
                .unwrap_or_default()
            {
                let destination_section_id = transition.destination().into();
                if visited_sections.contains(&destination_section_id) {
                    continue;
                }

                // TODO: draw switches
                let switches = transition.required_switch_changes();
                let relative_angle =
                    switches
                        .into_iter()
                        .fold(0.0, |acc, (_, switch_state)| match switch_state {
                            SwitchState::Left => acc - f32::consts::FRAC_PI_4,
                            SwitchState::Right => acc,
                        });

                let new_render_result = new_render_result.rotated(relative_angle);

                sections_to_visit.push_back(SectionVisit {
                    section_id: destination_section_id,
                    render_result: new_render_result,
                    section_direction: match transition.destination_section_end() {
                        SectionEnd::Start => Direction::Backward,
                        SectionEnd::End => Direction::Forward,
                    },
                });
            }
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
