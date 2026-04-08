use gpui::{
    AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, Render, Subscription, Window,
};
use gpui_component::{
    dock::{Panel, PanelEvent},
    table::TableState,
};
use liketrain_core::{
    SwitchId,
    ui::{UiEvent, UiSwitchEvent},
};

use crate::{
    controller::ControllerUiWrapper,
    window::controls::{
        panel_type::ControlsWindowPanelType,
        panels::switch::table::{SwitchesTableData, SwitchesTableDelegate},
    },
};

mod table;

pub struct SwitchesPanel {
    focus_handle: FocusHandle,

    table_state: Entity<TableState<SwitchesTableDelegate>>,

    _subscriptions: Vec<Subscription>,
}

impl SwitchesPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let table_state = cx.new(|cx| {
            let controller_state = ControllerUiWrapper::state(cx).read(cx);
            TableState::new(
                SwitchesTableDelegate::new(controller_state.switch_states().map(
                    |(switch_id, &state)| SwitchesTableData {
                        id: switch_id.clone(),
                        state,
                    },
                )),
                window,
                cx,
            )
        });

        let _subscriptions = vec![cx.subscribe(
            &ControllerUiWrapper::state(cx).clone(),
            |this, _, evt, cx| match evt {
                UiEvent::UiSwitchEvent(switch_event) => match switch_event {
                    UiSwitchEvent::SetState { id, .. } => this.update_switch_state(id, cx),
                },
                _ => {}
            },
        )];

        Self {
            focus_handle: cx.focus_handle(),
            table_state,
            _subscriptions,
        }
    }

    fn update_switch_state(&self, switch_id: &SwitchId, cx: &mut Context<Self>) {
        self.table_state.update(cx, |state, cx| {
            state.delegate_mut().update_switch_state(switch_id, cx);
            cx.notify();
        });
        cx.notify();
    }
}

impl Render for SwitchesPanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        self.table_state.clone()
    }
}

impl EventEmitter<PanelEvent> for SwitchesPanel {}
impl Focusable for SwitchesPanel {
    fn focus_handle(&self, _cx: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for SwitchesPanel {
    fn panel_name(&self) -> &'static str {
        ControlsWindowPanelType::Switches.panel_name()
    }

    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        "Switches"
    }

    fn closable(&self, _cx: &gpui::App) -> bool {
        false
    }
}
