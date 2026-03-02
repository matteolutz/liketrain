use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq)]
#[serde(transparent)]
pub struct LayoutColor([u8; 3]);

impl LayoutColor {
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        LayoutColor([r, g, b])
    }

    pub fn r(&self) -> u8 {
        self.0[0]
    }

    pub fn g(&self) -> u8 {
        self.0[1]
    }

    pub fn b(&self) -> u8 {
        self.0[2]
    }
}

impl From<LayoutColor> for gpui::Rgba {
    fn from(value: LayoutColor) -> Self {
        let rgb_value = (value.r() as u32) << 16 | (value.g() as u32) << 8 | (value.b() as u32);
        gpui::rgb(rgb_value)
    }
}
