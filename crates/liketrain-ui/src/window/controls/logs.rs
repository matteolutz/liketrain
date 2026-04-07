use gpui::{IntoElement, ParentElement, RenderOnce, Styled, div};
use gpui_component::{scroll::ScrollableElement, v_flex};

use crate::controller::ControllerUiWrapper;

#[derive(IntoElement)]
pub struct LogsTab;

impl RenderOnce for LogsTab {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl gpui::IntoElement {
        v_flex().size_full().overflow_y_scrollbar().children(
            ControllerUiWrapper::state(cx)
                .read(cx)
                .logs()
                .iter()
                .map(|log| div().child(log.message.clone())),
        )
    }
}
