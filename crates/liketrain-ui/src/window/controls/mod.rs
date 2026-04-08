use gpui::{
    AppContext, Context, Entity, ParentElement, Render, Styled, Subscription, Window, div,
    prelude::FluentBuilder,
};
use gpui_component::{
    Disableable,
    button::Button,
    tab::{Tab, TabBar},
};

use crate::{
    app_ext::GpuiContextExtension,
    controller::ControllerUiWrapper,
    window::controls::{logs::LogsTab, section::SectionsTab},
};

mod logs;
mod section;

pub struct ControlsWindow {
    selected_tab: usize,

    sections_tab: Entity<SectionsTab>,

    _subscriptions: Vec<Subscription>,
}

impl ControlsWindow {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let _subscriptions = vec![cx.observe_and_notify(&ControllerUiWrapper::state(cx).clone())];

        let sections_tab = cx.new(|cx| SectionsTab::new(window, cx));

        Self {
            selected_tab: 0,
            sections_tab,
            _subscriptions,
        }
    }
}

impl Render for ControlsWindow {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                Button::new("start")
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
                TabBar::new("tabs")
                    .selected_index(self.selected_tab)
                    .on_click(cx.listener(|this, selected_idx, _, cx| {
                        this.selected_tab = *selected_idx;
                        cx.notify();
                    }))
                    .child(Tab::new().label("Sections"))
                    .child(Tab::new().label("Switches"))
                    .child(Tab::new().label("Logs")),
            )
            .when(true, |this| match self.selected_tab {
                0 => this.child(self.sections_tab.clone()),
                2 => this.child(LogsTab),
                _ => this,
            })
    }
}
