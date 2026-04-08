use gpui::{
    AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, ParentElement, Render,
    Styled, div, prelude::FluentBuilder,
};
use gpui_component::dock::{Panel, PanelEvent};

use crate::{
    controller::ControllerUiWrapper, layout::LayoutRenderer,
    window::controls::panel_type::ControlsWindowPanelType,
};

pub struct LayoutPanel {
    focus_handle: FocusHandle,

    layout_renderer: Option<Entity<LayoutRenderer>>,
}

impl LayoutPanel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let layout_renderer = ControllerUiWrapper::layout(cx)
            .cloned()
            .map(|layout| cx.new(|cx| layout.renderer(cx)));

        Self {
            focus_handle: cx.focus_handle(),
            layout_renderer,
        }
    }
}

impl Render for LayoutPanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> impl gpui::IntoElement {
        div()
            .size_full()
            .when_some(self.layout_renderer.clone(), |this, layout_renderer| {
                this.child(layout_renderer)
            })
    }
}

impl EventEmitter<PanelEvent> for LayoutPanel {}
impl Focusable for LayoutPanel {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for LayoutPanel {
    fn panel_name(&self) -> &'static str {
        ControlsWindowPanelType::Layout.panel_name()
    }

    fn title(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> impl gpui::IntoElement {
        "Track Layout"
    }

    fn inner_padding(&self, _cx: &gpui::App) -> bool {
        false
    }

    fn closable(&self, _cx: &gpui::App) -> bool {
        false
    }
}
