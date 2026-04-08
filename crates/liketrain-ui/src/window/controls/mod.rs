use gpui::{AppContext, Context, Entity, ParentElement, Render, Styled, Subscription, Window, div};
use gpui_component::dock::{DockArea, DockPlacement, PanelStyle};

use crate::{
    app_ext::GpuiContextExtension, controller::ControllerUiWrapper,
    window::controls::panel_type::ControlsWindowPanelType,
};

mod panel_type;
mod panels;

pub struct ControlsWindow {
    dock_area: Entity<DockArea>,

    _subscriptions: Vec<Subscription>,
}

impl ControlsWindow {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let _subscriptions = vec![cx.observe_and_notify(&ControllerUiWrapper::state(cx).clone())];

        let dock_area = cx.new(|cx| {
            let mut da =
                DockArea::new("dock-area", Some(5), window, cx).panel_style(PanelStyle::TabBar);

            da.add_panel(
                ControlsWindowPanelType::Sections.build_panel_view(window, cx),
                DockPlacement::Center,
                None,
                window,
                cx,
            );
            da.add_panel(
                ControlsWindowPanelType::Switches.build_panel_view(window, cx),
                DockPlacement::Center,
                None,
                window,
                cx,
            );
            da.add_panel(
                ControlsWindowPanelType::Trains.build_panel_view(window, cx),
                DockPlacement::Center,
                None,
                window,
                cx,
            );
            da.add_panel(
                ControlsWindowPanelType::Layout.build_panel_view(window, cx),
                DockPlacement::Center,
                None,
                window,
                cx,
            );

            da.add_panel(
                ControlsWindowPanelType::Logs.build_panel_view(window, cx),
                DockPlacement::Right,
                None,
                window,
                cx,
            );

            da.add_panel(
                ControlsWindowPanelType::Controller.build_panel_view(window, cx),
                DockPlacement::Bottom,
                None,
                window,
                cx,
            );

            da
        });

        Self {
            dock_area,
            _subscriptions,
        }
    }
}

impl Render for ControlsWindow {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div().size_full().child(self.dock_area.clone())
    }
}
