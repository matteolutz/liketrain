use gpui::{
    AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, Render, Subscription, Window,
};
use gpui_component::{
    dock::{Panel, PanelEvent},
    table::TableState,
};
use liketrain_core::{
    SectionId,
    ui::{UiEvent, UiSectionEvent},
};

use crate::{
    controller::ControllerUiWrapper,
    window::controls::{
        panel_type::ControlsWindowPanelType,
        panels::section::table::{SectionsTableData, SectionsTableDelegate},
    },
};

mod table;

pub struct SectionsPanel {
    focus_handle: FocusHandle,

    table_state: Entity<TableState<SectionsTableDelegate>>,

    _subscriptions: Vec<Subscription>,
}

impl SectionsPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let table_state = cx.new(|cx| {
            let controller_state = ControllerUiWrapper::state(cx).read(cx);
            TableState::new(
                SectionsTableDelegate::new(controller_state.section_states().map(
                    |(section_id, state)| SectionsTableData {
                        id: section_id,
                        occupant: state.occupant,
                        reservation: state.reserved_by,
                        queue: state.queue.iter().copied().collect(),
                        power: state.power,
                    },
                )),
                window,
                cx,
            )
        });

        let _subscriptions = vec![cx.subscribe(
            &ControllerUiWrapper::state(cx).clone(),
            |this, _, evt, cx| match evt {
                UiEvent::UiSectionEvent(section_event) => match section_event {
                    &UiSectionEvent::SetPower { section_id, .. } => {
                        this.update_section_power(section_id, cx)
                    }
                    &UiSectionEvent::Occupied { section_id, .. } => {
                        this.update_section_occupant(section_id, cx)
                    }
                    &UiSectionEvent::Reserved { section_id, .. } => {
                        this.update_section_reservation(section_id, cx)
                    }
                    &UiSectionEvent::QueueDequeued { section_id, .. }
                    | &UiSectionEvent::QueueEnqueued { section_id, .. } => {
                        this.update_section_queue(section_id, cx)
                    }
                    UiSectionEvent::HardwareSectionEvent(_) => {}
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

    fn update_section_power(&self, section_id: SectionId, cx: &mut Context<Self>) {
        self.table_state.update(cx, |state, cx| {
            state.delegate_mut().update_section_power(section_id, cx);
            cx.notify();
        });
        cx.notify();
    }

    fn update_section_occupant(&self, section_id: SectionId, cx: &mut Context<Self>) {
        self.table_state.update(cx, |state, cx| {
            state.delegate_mut().update_section_occupant(section_id, cx);
            cx.notify();
        });
        cx.notify();
    }

    fn update_section_reservation(&self, section_id: SectionId, cx: &mut Context<Self>) {
        self.table_state.update(cx, |state, cx| {
            state
                .delegate_mut()
                .update_section_reservation(section_id, cx);
            cx.notify();
        });
        cx.notify();
    }

    fn update_section_queue(&self, section_id: SectionId, cx: &mut Context<Self>) {
        self.table_state.update(cx, |state, cx| {
            state.delegate_mut().update_section_queue(section_id, cx);
            cx.notify();
        });
        cx.notify();
    }
}

impl Render for SectionsPanel {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        self.table_state.clone()
    }
}

impl EventEmitter<PanelEvent> for SectionsPanel {}
impl Focusable for SectionsPanel {
    fn focus_handle(&self, _cx: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Panel for SectionsPanel {
    fn panel_name(&self) -> &'static str {
        ControlsWindowPanelType::Sections.panel_name()
    }

    fn title(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl gpui::IntoElement {
        "Sections"
    }

    fn closable(&self, _cx: &gpui::App) -> bool {
        false
    }
}
