use gpui::{Pixels, Point, point, px};

pub trait Vec2Ext {
    fn to_point(&self) -> Point<Pixels>;
    fn from_point(point: Point<Pixels>) -> Self;

    fn to_normalized_point(&self) -> Point<Pixels>;
}

impl Vec2Ext for vek::Vec2<f32> {
    fn to_point(&self) -> Point<Pixels> {
        point(px(self.x), px(self.y))
    }

    fn from_point(point: Point<Pixels>) -> Self {
        Self::new(point.x.as_f32(), point.y.as_f32())
    }

    fn to_normalized_point(&self) -> Point<Pixels> {
        let normalized = self.normalized();
        normalized.to_point()
    }
}
