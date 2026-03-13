use std::{collections::HashMap, f32, rc::Rc};

use gpui::{
    Bounds, Context, InteractiveElement, MouseDownEvent, MouseMoveEvent, ParentElement,
    PathBuilder, Pixels, Point, Render, ScrollWheelEvent, Styled, Window, canvas, div, point, px,
};
use liketrain_core::{SectionId, SwitchId, Track};
use serde::{Deserialize, Serialize};
use vek::Vec2;

use crate::{
    app_ext::GpuiContextExtension,
    layout::{
        camera::LayoutCamera,
        color::LayoutColor,
        switch::{SwitchResolution, SwitchResolutionEnd},
        vec::Vec2Ext,
    },
};

mod camera;
mod color;
mod switch;
mod vec;

const SECTION_STROKE_WIDTH: f32 = 1.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSectionGeometry {
    pub to: Point<Pixels>,
    pub geo_type: LayoutSectionGeometryType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutSectionGeometryType {
    Straight,
    Arc {
        angle: f32,

        #[serde(default)]
        sweep: Option<bool>,
    },
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
    ) -> Point<Pixels> {
        let gpui_color: gpui::Rgba = color.into();

        let from = camera.project(from);
        let to = camera.project(self.to);
        let stroke_width = camera.scale(SECTION_STROKE_WIDTH);

        let path = match self.geo_type {
            LayoutSectionGeometryType::Straight => {
                let mut path_builder = PathBuilder::stroke(px(stroke_width));
                path_builder.move_to(from);

                path_builder.line_to(to);

                let path = path_builder.build().unwrap();
                path
            }
            LayoutSectionGeometryType::Arc { angle, sweep } => {
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
                    sweep.unwrap_or(angle > f32::consts::PI),
                    to,
                );

                let path = path_builder.build().unwrap();
                path
            }
        };

        let center = path.bounds.center();

        window.paint_path(path, gpui_color);
        center
    }
}

#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
enum LayoutSectionEnd {
    #[default]
    Default,

    Dead,

    Arrow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSection {
    from: Point<Pixels>,
    geometries: Vec<LayoutSectionGeometry>,

    #[serde(default)]
    from_end: LayoutSectionEnd,

    #[serde(default)]
    to_end: LayoutSectionEnd,

    color: LayoutColor,
}

impl LayoutSection {
    fn to(&self) -> Point<Pixels> {
        self.geometries.last().map(|g| g.to).unwrap_or(self.from)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    sections: HashMap<SectionId, LayoutSection>,
}

#[derive(Debug, Clone)]
pub struct ResolvedLayout {
    layout: Layout,
    switch_resolutions: HashMap<SwitchId, SwitchResolution>,
}

impl ResolvedLayout {
    fn render_switch(&self, switch: &SwitchResolution, camera: &LayoutCamera, window: &mut Window) {
        if let Some(center_point) = switch.center_point() {
            let points = [
                switch.get(SwitchResolutionEnd::From).ok().copied(),
                switch.get(SwitchResolutionEnd::ToLeft).ok().copied(),
                switch.get(SwitchResolutionEnd::ToRight).ok().copied(),
            ];

            for p in points {
                let Some(p) = p else {
                    continue;
                };

                let mut path_builder = PathBuilder::stroke(px(camera.scale(SECTION_STROKE_WIDTH)));
                path_builder.move_to(camera.project(center_point));
                path_builder.line_to(camera.project(p));

                let path = path_builder.build().unwrap();
                window.paint_path(path, gpui::green());
            }
        } else {
            // TODO
        }
    }

    fn render_canvas(&self, _track: &Track, camera: &LayoutCamera, window: &mut Window) {
        // draw sections
        for (_, layout_section) in self.layout.sections.iter() {
            layout_section
                .geometries
                .iter()
                .fold(layout_section.from, |from, geo| {
                    geo.render(from, layout_section.color, camera, window);
                    geo.to
                });
        }

        // draw switches
        for (_, resolution) in self.switch_resolutions.iter() {
            self.render_switch(resolution, camera, window);
        }
    }

    pub fn renderer(&self, track: &Rc<Track>, _cx: &mut Context<LayoutRenderer>) -> LayoutRenderer {
        LayoutRenderer {
            layout: self.clone(),
            track: track.clone(),
            camera: LayoutCamera::new(Bounds::default()),
            last_mouse_pos: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LayoutRenderConfig {
    /// The zoom level to render at
    pub zoom: f32,

    /// The center point to render at
    pub center: Point<Pixels>,
}

pub struct LayoutRenderer {
    layout: ResolvedLayout,
    track: Rc<Track>,

    last_mouse_pos: Option<Point<Pixels>>,

    camera: LayoutCamera,
}

impl Render for LayoutRenderer {
    fn render(
        &mut self,
        _: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div()
            .size_full()
            .relative()
            .on_scroll_wheel(cx.listener(|this, evt: &ScrollWheelEvent, _, cx| {
                let delta_y = evt.delta.pixel_delta(px(1.0)).y.as_f32();
                this.camera.zoom = (this.camera.zoom + (delta_y / 4.0)).max(0.0);
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

                this.camera.position -= delta / this.camera.zoom;

                cx.notify();
            }))
            .child(
                canvas(
                    cx.prepaint_canvas(|this, bounds, _, cx| {
                        this.camera.screen_bounds = bounds;
                        cx.notify();
                    }),
                    cx.paint_canvas(|this, _, _, window, _| {
                        this.layout.render_canvas(&this.track, &this.camera, window);
                    }),
                )
                .absolute()
                .top_0()
                .left_0()
                .bg(gpui::black())
                .cursor_crosshair()
                .size_full(),
            )
            .child(
                div().absolute().top_0().left_0().size_full().children(
                    self.layout
                        .switch_resolutions
                        .iter()
                        .filter_map(|(id, switch)| switch.center_point().map(|cp| (id, switch, cp)))
                        .map(|(switch_id, _, cp)| {
                            /*
                            let from_point =
                                switch.get(SwitchResolutionEnd::From).ok().copied().unwrap(); // safe because center_point is derived from from/to points

                            let normal_vec = Vec2::from_point(cp - from_point)
                                .normalized()
                                .rotated_z(f32::consts::FRAC_PI_2)
                                .normalized();

                            let offset = normal_vec * 2.0;

                            let point = self.camera.project(cp + offset.to_point());
                            */

                            let point = self.camera.project(cp);

                            let container_size = self.camera.scale(px(3.0));
                            let container_rounding = self.camera.scale(px(1.0));
                            let text_size = self.camera.scale(px(2.0));

                            div()
                                .flex()
                                .justify_center()
                                .items_center()
                                .size(container_size)
                                .bg(gpui::white())
                                .rounded(container_rounding)
                                .absolute()
                                .text_size(text_size)
                                .top(point.y)
                                .left(point.x)
                                .child(switch_id.to_string())
                        }),
                ),
            )
    }
}
