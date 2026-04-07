use std::{collections::HashMap, sync::mpsc, time::Duration};

use gpui::{App, AppContext, BorrowAppContext, Context, Entity, EventEmitter, Global, Task};
use liketrain_core::{
    Controller, ControllerConfig, ControllerError, SectionId, SwitchId, SwitchState, Track,
    TrainData, TrainId,
    comm::ControllerHardwareCommunication,
    ui::{UiCommand, UiEvent, UiSectionEvent, UiSwitchEvent, UiTrainEvent},
};

mod section;
pub use section::*;

#[derive(Default)]
pub struct ControllerUiWrapperState {
    track: Track,
    trains: HashMap<TrainId, TrainData>,

    section_states: HashMap<SectionId, UiSectionState>,
    switch_states: HashMap<SwitchId, SwitchState>,
}

impl EventEmitter<UiEvent> for ControllerUiWrapperState {}

impl ControllerUiWrapperState {
    pub fn section_states(&self) -> impl Iterator<Item = (SectionId, &UiSectionState)> {
        self.section_states.iter().map(|(id, state)| (*id, state))
    }

    pub fn section_state(&self, section_id: SectionId) -> Option<&UiSectionState> {
        self.section_states.get(&section_id)
    }

    pub fn train(&self, train_id: TrainId) -> Option<&TrainData> {
        self.trains.get(&train_id)
    }
}

impl ControllerUiWrapperState {
    fn from_controller(controller: &Controller) -> Self {
        Self {
            track: controller.track().clone(),
            trains: controller
                .trains()
                .map(|(id, train)| (id, train.data().clone()))
                .collect(),
            switch_states: controller.switch_states().collect(),
            section_states: controller
                .section_states()
                .map(|(id, state)| {
                    let reservation = controller.section_reservation(id);
                    let queue = controller.section_queue(id).cloned().unwrap_or_default();

                    (
                        id,
                        UiSectionState {
                            power: state.power(),
                            occupant: state.occupant(),
                            reserved_by: reservation,
                            queue,
                        },
                    )
                })
                .collect(),
        }
    }

    fn handle_section_event(&mut self, section_event: UiSectionEvent, cx: &mut Context<Self>) {
        match section_event {
            UiSectionEvent::Occupied {
                section_id,
                train_id,
            } => {
                self.section_states.entry(section_id).or_default().occupant = train_id;
                cx.notify();
            }
            UiSectionEvent::Reserved {
                section_id,
                train_id,
            } => {
                self.section_states
                    .entry(section_id)
                    .or_default()
                    .reserved_by = train_id;
                cx.notify();
            }
            UiSectionEvent::SetPower { section_id, power } => {
                self.section_states.entry(section_id).or_default().power = power;
                cx.notify();
            }
            UiSectionEvent::QueueEnqueued {
                section_id,
                train_id,
            } => {
                self.section_states
                    .entry(section_id)
                    .or_default()
                    .queue
                    .push_back(train_id);
                cx.notify();
            }
            UiSectionEvent::QueueDequeued {
                section_id,
                train_id,
            } => {
                self.section_states
                    .entry(section_id)
                    .or_default()
                    .queue
                    .retain(|id| id != &train_id);
                cx.notify();
            }
            UiSectionEvent::HardwareSectionEvent(_) => {}
        }
    }

    fn handle_switch_event(&mut self, switch_event: UiSwitchEvent, cx: &mut Context<Self>) {
        match switch_event {
            UiSwitchEvent::SetState { id, state } => {
                self.switch_states.insert(id, state);
                cx.notify();
            }
        }
    }

    fn handle_train_event(&mut self, _train_event: UiTrainEvent, _cx: &mut Context<Self>) {
        // TODO
    }

    fn handle_event(&mut self, event: UiEvent, cx: &mut Context<Self>) {
        // update own state based on event
        match event.clone() {
            UiEvent::UiSectionEvent(section_event) => self.handle_section_event(section_event, cx),
            UiEvent::UiSwitchEvent(switch_event) => self.handle_switch_event(switch_event, cx),
            UiEvent::UiTrainEvent(train_event) => self.handle_train_event(train_event, cx),
        }

        // and then also emit the event to any listeners
        cx.emit(event);
    }
}

pub struct ControllerUiWrapper {
    // has to be Option because we need to `.take()` it out when starting
    controller: Option<Controller>,
    controller_state: Entity<ControllerUiWrapperState>,

    command_tx: crossbeam::channel::Sender<UiCommand>,

    task: Option<Task<()>>,
}

impl ControllerUiWrapper {
    pub fn new(
        cx: &mut App,
        config: ControllerConfig,
        hardware_comm: impl ControllerHardwareCommunication + 'static,
    ) -> Self {
        let (event_tx, event_rx) = mpsc::channel();
        let (command_tx, command_rx) = crossbeam::channel::unbounded();

        let controller = Controller::new(config, hardware_comm, event_tx, command_rx);

        let controller_state = cx.new(|_| ControllerUiWrapperState::from_controller(&controller));

        let _task = cx.spawn({
            let controller_state = controller_state.clone();
            async move |cx| {
                loop {
                    cx.background_executor()
                        .timer(Duration::from_millis(16))
                        .await;

                    for event in event_rx.try_iter() {
                        controller_state.update(cx, |this, cx| this.handle_event(event, cx));
                    }
                }
            }
        });

        Self {
            controller: Some(controller),
            command_tx,
            controller_state,
            task: None,
        }
    }

    pub fn can_start(cx: &App) -> bool {
        cx.global::<Self>().controller.is_some()
    }

    pub fn start(cx: &mut App) -> Result<(), ControllerError> {
        cx.update_global(|this: &mut Self, _| {
            let controller = this
                .controller
                .take()
                .expect("controller should be present. Please only call start() once");
            controller.start()
        })
    }

    pub fn exec(command: impl Into<UiCommand>, cx: &App) {
        let command = command.into();
        let _ = cx.global::<Self>().command_tx.send(command);
    }

    pub fn state(cx: &App) -> &Entity<ControllerUiWrapperState> {
        &cx.global::<Self>().controller_state
    }
}

impl Global for ControllerUiWrapper {}
impl EventEmitter<UiEvent> for ControllerUiWrapper {}
