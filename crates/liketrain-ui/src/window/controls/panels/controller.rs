use gpui::{Context, EventEmitter, FocusHandle, Focusable, ParentElement, Render, Styled};
use gpui_component::{
    Disableable, IconName,
    button::{Button, ButtonVariant, ButtonVariants},
    dock::{Panel, PanelEvent},
    h_flex,
};

use crate::{
    controller::ControllerUiWrapper, window::controls::panel_type::ControlsWindowPanelType,
};

pub struct ControllerPanel {
    focus_handle: FocusHandle,
}

impl ControllerPanel {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Render for ControllerPanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut Context<Self>,
    ) -> impl gpui::IntoElement {
        h_flex()
            .size_full()
            .p_2()
            .gap_2()
            .child(
                Button::new("start")
                    .icon(IconName::Play)
                    .with_variant(ButtonVariant::Success)
                    .label("Start")
                    .disabled(!ControllerUiWrapper::can_start(cx))
                    .on_click(|_, _, cx| {
                        if !ControllerUiWrapper::can_start(cx) {
                            return;
                        }

                        ControllerUiWrapper::start(cx);
                    }),
            )
            .child(
                Button::new("pause")
                    .icon(IconName::Pause)
                    .disabled(ControllerUiWrapper::can_start(cx))
                    .label("Pause"),
            )
            .child(
                Button::new("stop")
                    .disabled(ControllerUiWrapper::can_start(cx))
                    .with_variant(ButtonVariant::Danger)
                    .label("Stop"),
            )
    }
}

impl EventEmitter<PanelEvent> for ControllerPanel {}
impl Focusable for ControllerPanel {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for ControllerPanel {
    fn panel_name(&self) -> &'static str {
        ControlsWindowPanelType::Controller.panel_name()
    }

    fn title(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut Context<Self>,
    ) -> impl gpui::IntoElement {
        "Controller"
    }

    fn closable(&self, _cx: &gpui::App) -> bool {
        false
    }
}
