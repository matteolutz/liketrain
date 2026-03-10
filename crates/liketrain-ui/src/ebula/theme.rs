use gpui::{Hsla, Pixels, px};

#[derive(Debug, Clone)]
pub struct EbulaTheme {
    pub background: Hsla,
    pub foreground: Hsla,

    pub border_width: Pixels,
}

impl EbulaTheme {
    pub fn default_light() -> Self {
        Self {
            background: gpui::white(),
            foreground: gpui::black(),
            border_width: px(2.0),
        }
    }

    pub fn default_dark() -> Self {
        Self {
            background: gpui::black(),
            foreground: gpui::white(),
            border_width: px(2.0),
        }
    }
}
