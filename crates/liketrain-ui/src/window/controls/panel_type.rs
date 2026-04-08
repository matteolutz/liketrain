use std::sync::Arc;

use gpui::{App, AppContext, Window};
use gpui_component::dock::PanelView;

use crate::window::controls::panels::{
    ControllerPanel, LayoutPanel, LogsPanel, SectionsPanel, SwitchesPanel, TrainsPanel,
};

#[derive(Debug, Copy, Clone, strum::EnumIter)]
pub enum ControlsWindowPanelType {
    Sections,
    Switches,
    Trains,
    Controller,
    Layout,
    Logs,
}

impl ControlsWindowPanelType {
    pub fn panel_name(&self) -> &'static str {
        match self {
            ControlsWindowPanelType::Sections => "liketrain-sections",
            ControlsWindowPanelType::Switches => "liketrain-switches",
            ControlsWindowPanelType::Trains => "liketrain-trains",
            ControlsWindowPanelType::Controller => "liketrain-controller",
            ControlsWindowPanelType::Layout => "liketrain-layout",
            ControlsWindowPanelType::Logs => "liketrain-logs",
        }
    }

    pub fn build_panel_view(self, window: &mut Window, cx: &mut App) -> Arc<dyn PanelView> {
        match self {
            Self::Sections => Arc::new(cx.new(|cx| SectionsPanel::new(window, cx))),
            Self::Switches => Arc::new(cx.new(|cx| SwitchesPanel::new(window, cx))),
            Self::Trains => Arc::new(cx.new(|cx| TrainsPanel::new(window, cx))),
            Self::Controller => Arc::new(cx.new(ControllerPanel::new)),
            Self::Layout => Arc::new(cx.new(LayoutPanel::new)),
            Self::Logs => Arc::new(cx.new(LogsPanel::new)),
        }
    }
}
