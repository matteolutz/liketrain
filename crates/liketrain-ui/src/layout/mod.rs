use std::{collections::HashMap, f32, rc::Rc};

use gpui::{
    Bounds, IntoElement, PathBuilder, Pixels, Point, RenderOnce, Styled, Window, canvas, fill,
    point, px, size,
};
use liketrain_core::{SectionId, SwitchId, Track};
use serde::{Deserialize, Serialize, de::Visitor};
use strum::IntoEnumIterator;
use vek::Vec2;

use crate::layout::{
    camera::LayoutCamera,
    color::LayoutColor,
    switch::{SwitchResolution, SwitchResolutionEnd, SwitchResolutionState},
    vec::Vec2Ext,
};

mod camera;
mod color;
mod switch;
mod vec;

const SECTION_STROKE_WIDTH: f32 = 1.0;

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
pub struct LayoutSectionGeometry {
    pub to: Point<Pixels>,
    pub geo_type: LayoutSectionGeometryType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutSectionGeometryType {
    Straight,
    Arc { angle: f32 },
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
        from: Point<Pixels>,
        color: LayoutColor,
        camera: &LayoutCamera,
        window: &mut Window,
    ) {
        let gpui_color: gpui::Rgba = color.into();

        let from = camera.project(from);
        let to = camera.project(self.to);
        let stroke_width = camera.scale(SECTION_STROKE_WIDTH);

        match self.geo_type {
            LayoutSectionGeometryType::Straight => {
                let mut path_builder = PathBuilder::stroke(px(stroke_width));
                path_builder.move_to(from);

                path_builder.line_to(to);

                let path = path_builder.build().unwrap();
                window.paint_path(path, gpui_color);
            }
            LayoutSectionGeometryType::Arc { angle } => {
                let mut path_builder = PathBuilder::stroke(px(stroke_width));
                path_builder.move_to(from);

                let offset = Vec2::from_point(to - from);

                let actual_angle = offset.x.atan2(offset.y);

                let angle = angle.abs().to_radians() * actual_angle.signum();

                let chord_len = (to - from).magnitude() as f32;
                let abs_angle = angle.abs();

                let r = chord_len / (2.0 * (abs_angle / 2.0).sin());

                path_builder.arc_to(
                    point(px(r), px(r)),
                    px(0.0),
                    false,
                    actual_angle > f32::consts::PI,
                    to,
                );

                let path = path_builder.build().unwrap();
                window.paint_path(path, gpui_color);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSection {
    from: Point<Pixels>,
    geometries: Vec<LayoutSectionGeometry>,

    color: LayoutColor,
}

impl LayoutSection {
    fn to(&self) -> Point<Pixels> {
        self.geometries.last().map(|g| g.to).unwrap_or(self.from)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    sections: HashMap<LayoutSectionId, LayoutSection>,
}

#[derive(Debug, Clone)]
pub struct ResolvedLayout {
    layout: Layout,
    switch_resolutions: HashMap<SwitchId, SwitchResolution>,
}

impl ResolvedLayout {
    fn render(
        &self,
        _track: &Track,
        config: LayoutRenderConfig,
        bounds: Bounds<Pixels>,
        window: &mut Window,
    ) {
        let camera = LayoutCamera::new(bounds)
            .with_zoom(config.zoom)
            .with_center(config.center);

        // draw sections
        for (_, layout_section) in self.layout.sections.iter() {
            layout_section
                .geometries
                .iter()
                .fold(layout_section.from, |from, geo| {
                    geo.render(from, layout_section.color, &camera, window);
                    geo.to
                });
        }

        // draw switches
        for (_, resolution) in self.switch_resolutions.iter() {
            for end in SwitchResolutionEnd::iter() {
                let SwitchResolutionState::Resolved(point) = resolution.get(end) else {
                    continue;
                };

                let point = camera.project(*point);
                let size = camera.scale_size(size(px(2.0), px(2.0)));

                window.paint_quad(fill(Bounds::centered_at(point, size), gpui::green()));
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
    /// The zoom level to render at
    pub zoom: f32,

    /// The center point to render at
    pub center: Point<Pixels>,
}

#[derive(IntoElement)]
pub struct LayoutRenderer {
    layout: ResolvedLayout,
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
