use gpui::{Context, EventEmitter, FocusHandle, Focusable, ParentElement, Render, Styled, div};
use gpui_component::{
    dock::{Panel, PanelEvent},
    scroll::ScrollableElement,
    v_flex,
};

use crate::{
    controller::ControllerUiWrapper, window::controls::panel_type::ControlsWindowPanelType,
};

pub struct LogsPanel {
    focus_handle: FocusHandle,
}

impl LogsPanel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Render for LogsPanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> impl gpui::IntoElement {
        v_flex().size_full().overflow_y_scrollbar().children(
            ControllerUiWrapper::state(cx)
                .read(cx)
                .logs()
                .iter()
                .map(|log| div().child(log.message.clone())),
        )
    }
}

impl EventEmitter<PanelEvent> for LogsPanel {}
impl Focusable for LogsPanel {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for LogsPanel {
    fn panel_name(&self) -> &'static str {
        ControlsWindowPanelType::Logs.panel_name()
    }

    fn title(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> impl gpui::IntoElement {
        "Logs"
    }

    fn closable(&self, _cx: &gpui::App) -> bool {
        false
    }
}
