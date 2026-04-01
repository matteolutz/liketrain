use gpui::{
    AppContext, Context, Entity, ParentElement, Render, Styled, Subscription, Window, div,
    prelude::FluentBuilder,
};
use gpui_component::{
    button::Button,
    tab::{Tab, TabBar},
};

use crate::{controller::ControllerUiWrapper, window::controls::section::SectionsTab};

mod section;

pub struct ControlsWindow {
    selected_tab: usize,

    sections_tab: Entity<SectionsTab>,

    _subscriptions: Vec<Subscription>,
}

impl ControlsWindow {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let _subscriptions = vec![cx.subscribe(
            &ControllerUiWrapper::event_emitter(cx).clone(),
            |_, _, _, cx| {
                cx.notify();
            },
        )];

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
            .child(div().text_xl().child("Controls"))
            .child(
                div().w_full().flex().p_2().child(
                    Button::new("start-button")
                        .label("Start")
                        .on_click(|_, _, _| {}),
                ),
            )
            .child(
                TabBar::new("tabs")
                    .selected_index(self.selected_tab)
                    .on_click(cx.listener(|this, selected_idx, _, cx| {
                        this.selected_tab = *selected_idx;
                        cx.notify();
                    }))
                    .child(Tab::new().label("Sections"))
                    .child(Tab::new().label("Switches")),
            )
            .when(true, |this| match self.selected_tab {
                0 => this.child(self.sections_tab.clone()),
                _ => this,
            })
    }
}
