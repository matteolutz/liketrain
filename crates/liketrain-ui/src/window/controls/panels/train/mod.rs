use gpui::{
    AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, Render, Subscription, Window,
};
use gpui_component::{
    dock::{Panel, PanelEvent},
    table::TableState,
};
use liketrain_core::{
    TrainId,
    ui::{UiEvent, UiTrainEvent},
};

use crate::{
    controller::ControllerUiWrapper,
    window::controls::{
        panel_type::ControlsWindowPanelType,
        panels::train::table::{TrainsTableData, TrainsTableDelegate},
    },
};

mod table;

pub struct TrainsPanel {
    focus_handle: FocusHandle,

    table_state: Entity<TableState<TrainsTableDelegate>>,

    _subscriptions: Vec<Subscription>,
}

impl TrainsPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let table_state = cx.new(|cx| {
            let controller_state = ControllerUiWrapper::state(cx).read(cx);
            TableState::new(
                TrainsTableDelegate::new(controller_state.trains().map(|(train_id, train)| {
                    TrainsTableData {
                        id: train_id,
                        current_section: train.current_section,
                    }
                })),
                window,
                cx,
            )
        });

        let _subscriptions = vec![cx.subscribe(
            &ControllerUiWrapper::state(cx).clone(),
            |this, _, evt, cx| match evt {
                UiEvent::UiTrainEvent(train_event) => match train_event {
                    &UiTrainEvent::EnteredSection { train_id, .. } => {
                        this.update_current_section(train_id, cx)
                    }
                    _ => {}
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

    fn update_current_section(&self, train_id: TrainId, cx: &mut Context<Self>) {
        self.table_state.update(cx, |state, cx| {
            state.delegate_mut().update_current_section(train_id, cx);
            cx.notify();
        });
        cx.notify();
    }
}

impl Render for TrainsPanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        self.table_state.clone()
    }
}

impl EventEmitter<PanelEvent> for TrainsPanel {}
impl Focusable for TrainsPanel {
    fn focus_handle(&self, _cx: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for TrainsPanel {
    fn panel_name(&self) -> &'static str {
        ControlsWindowPanelType::Trains.panel_name()
    }

    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        "Trains"
    }

    fn closable(&self, _cx: &gpui::App) -> bool {
        false
    }
}
