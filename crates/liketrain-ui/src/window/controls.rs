use gpui::{Context, ParentElement, Render, div};

pub struct ControlsWindow {}

impl ControlsWindow {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {}
    }
}

impl Render for ControlsWindow {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div().child("Controls")
    }
}
