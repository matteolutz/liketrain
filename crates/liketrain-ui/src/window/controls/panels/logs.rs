use gpui::{
    Context, EventEmitter, FocusHandle, Focusable, InteractiveElement, ParentElement, Render,
    ScrollHandle, StatefulInteractiveElement, Styled, Subscription, div,
};
use gpui_component::{
    ActiveTheme,
    dock::{Panel, PanelEvent},
    scroll::ScrollableElement,
    v_flex,
};

use crate::{
    controller::ControllerUiWrapper, window::controls::panel_type::ControlsWindowPanelType,
};

pub struct LogsPanel {
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,

    _subscriptions: Vec<Subscription>,
}

impl LogsPanel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let _subscriptions =
            vec![
                cx.observe(&ControllerUiWrapper::state(cx).clone(), |this, _, cx| {
                    this.scroll_handle.scroll_to_bottom();
                    cx.notify();
                }),
            ];

        Self {
            focus_handle: cx.focus_handle(),
            scroll_handle: ScrollHandle::new(),
            _subscriptions,
        }
    }
}

impl Render for LogsPanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> impl gpui::IntoElement {
        v_flex()
            .size_full()
            .id("logs")
            .track_scroll(&self.scroll_handle)
            .overflow_y_scrollbar()
            .children(
                ControllerUiWrapper::state(cx)
                    .read(cx)
                    .logs()
                    .iter()
                    .map(|log| {
                        div()
                            .font_family("JetBrains Mono")
                            .text_sm()
                            .p_2()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .child(log.message.clone())
                    }),
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
