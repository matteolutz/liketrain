use gpui::{AppContext, Context, Entity, Render, Subscription, Window};
use gpui_component::table::TableState;
use liketrain_core::{
    SectionId,
    hardware::event::HardwareSectionPower,
    ui::{UiEvent, UiSectionEvent},
};

use crate::{
    controller::ControllerUiWrapper,
    window::controls::section::table::{SectionsTableData, SectionsTableDelegate},
};

mod table;

pub struct SectionsTab {
    table_state: Entity<TableState<SectionsTableDelegate>>,

    _subscriptions: Vec<Subscription>,
}

impl SectionsTab {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let table_state = cx.new(|cx| {
            let controller = ControllerUiWrapper::controller(cx);
            TableState::new(
                SectionsTableDelegate::new(controller.track().sections().map(|(section_id, _)| {
                    SectionsTableData {
                        id: section_id,
                        occupant: None,
                        reservation: None,
                        queue: vec![],
                        power: controller
                            .section_state(section_id)
                            .map(|state| state.power())
                            .unwrap_or_default(),
                    }
                })),
                window,
                cx,
            )
        });

        let _subscriptions = vec![cx.subscribe(
            &ControllerUiWrapper::event_emitter(cx).clone(),
            |this, _, evt, cx| match evt {
                UiEvent::UiSectionEvent(section_event) => match section_event {
                    &UiSectionEvent::SetPower { section_id, power } => {
                        this.set_section_power(section_id, power, cx)
                    }
                    _ => {}
                },
                _ => {}
            },
        )];

        Self {
            table_state,
            _subscriptions,
        }
    }

    fn set_section_power(
        &self,
        section_id: SectionId,
        power: HardwareSectionPower,
        cx: &mut Context<Self>,
    ) {
        self.table_state.update(cx, |state, cx| {
            state.delegate_mut().set_section_power(section_id, power);
            cx.notify();
        });
        cx.notify();
    }
}

impl Render for SectionsTab {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        self.table_state.clone()
    }
}
