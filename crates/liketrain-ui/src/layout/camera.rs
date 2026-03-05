use std::ops::Mul;

use gpui::{Bounds, Pixels, Point, Size, point, px, size};

#[derive(Debug)]
pub struct LayoutCamera {
    pub position: Point<Pixels>,
    pub zoom: f32,

    pub screen_bounds: Bounds<Pixels>,
}

impl LayoutCamera {
    pub fn new(screen_bounds: impl Into<Bounds<Pixels>>) -> Self {
        Self {
            position: point(px(0.0), px(0.0)),
            zoom: 1.0,
            screen_bounds: screen_bounds.into(),
        }
    }

    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom = zoom;
        self
    }

    pub fn with_center(mut self, center: Point<Pixels>) -> Self {
        self.position = center;
        self
    }
}

impl LayoutCamera {
    /// Projects a point from world to screen coordinates.
    pub fn project(&self, world_point: Point<Pixels>) -> Point<Pixels> {
        let center = self.screen_bounds.center();

        let offset = world_point - self.position;
        let scaled = offset * self.zoom;

        point(center.x + scaled.x, center.y + scaled.y)
    }

    /// Unprojects a point from screen to world coordinates.
    pub fn unproject(&self, screen_point: Point<Pixels>) -> Point<Pixels> {
        let center = self.screen_bounds.center();

        let offset = screen_point - center;
        let scaled = offset / self.zoom;

        point(self.position.x + scaled.x, self.position.y + scaled.y)
    }

    pub fn scale_size(&self, size_to_scale: impl Into<Size<Pixels>>) -> Size<Pixels> {
        let size_to_scale = size_to_scale.into();
        size(
            self.scale(size_to_scale.width),
            self.scale(size_to_scale.height),
        )
    }

    pub fn scale<T>(&self, t: T) -> T
    where
        T: Mul<f32, Output = T>,
    {
        t * self.zoom
    }
}
