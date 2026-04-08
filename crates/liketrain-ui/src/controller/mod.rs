use std::{collections::HashMap, sync::mpsc, time::Duration};

use gpui::{
    App, AppContext, BorrowAppContext, Context, Entity, EventEmitter, Global, SharedString, Task,
};
use liketrain_core::{
    Controller, ControllerConfig, SectionId, SwitchId, SwitchState, Track, TrainId,
    comm::ControllerHardwareCommunication,
    hardware::event::SectionEventType,
    ui::{UiCommand, UiEvent, UiSectionEvent, UiSwitchEvent, UiTrainEvent},
};

mod section;
pub use section::*;

mod train;
pub use train::*;

use crate::layout::ResolvedLayout;

#[derive(Debug)]
pub enum ControllerUiLogType {
    UiEvent,
}

#[derive(Debug)]
pub struct ControllerUiLog {
    pub log_type: ControllerUiLogType,
    pub message: SharedString,
}

impl ControllerUiLog {
    fn ui_event(event: &UiEvent) -> Self {
        Self {
            log_type: ControllerUiLogType::UiEvent,
            message: SharedString::from(format!("{:?}", event)),
        }
    }
}

#[derive(Default)]
pub struct ControllerUiWrapperState {
    track: Track,
    trains: HashMap<TrainId, UiTrain>,

    section_states: HashMap<SectionId, UiSectionState>,
    switch_states: HashMap<SwitchId, SwitchState>,

    logs: Vec<ControllerUiLog>,
}

impl EventEmitter<UiEvent> for ControllerUiWrapperState {}

impl ControllerUiWrapperState {
    pub fn track(&self) -> &Track {
        &self.track
    }

    pub fn section_states(&self) -> impl Iterator<Item = (SectionId, &UiSectionState)> {
        self.section_states.iter().map(|(id, state)| (*id, state))
    }

    pub fn section_state(&self, section_id: SectionId) -> Option<&UiSectionState> {
        self.section_states.get(&section_id)
    }

    pub fn switch_states(&self) -> impl Iterator<Item = (&SwitchId, &SwitchState)> {
        self.switch_states.iter()
    }

    pub fn switch_state(&self, switch_id: &SwitchId) -> Option<&SwitchState> {
        self.switch_states.get(switch_id)
    }

    pub fn trains(&self) -> impl Iterator<Item = (TrainId, &UiTrain)> {
        self.trains.iter().map(|(id, train)| (*id, train))
    }

    pub fn train(&self, train_id: TrainId) -> Option<&UiTrain> {
        self.trains.get(&train_id)
    }

    pub fn logs(&self) -> &[ControllerUiLog] {
        &self.logs
    }
}

impl ControllerUiWrapperState {
    fn from_controller(controller: &Controller) -> Self {
        Self {
            track: controller.track().clone(),
            trains: controller
                .trains()
                .map(|(id, train)| (id, train.into()))
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
                            occupant: state.occupant().into(),
                            reserved_by: reservation,
                            queue,
                        },
                    )
                })
                .collect(),
            logs: Vec::new(),
        }
    }

    fn handle_section_event(&mut self, section_event: UiSectionEvent) {
        match section_event {
            UiSectionEvent::Occupied {
                section_id,
                train_id,
            } => {
                self.section_states.entry(section_id).or_default().occupant = train_id.into();
            }
            UiSectionEvent::Reserved {
                section_id,
                train_id,
            } => {
                self.section_states
                    .entry(section_id)
                    .or_default()
                    .reserved_by = train_id;
            }
            UiSectionEvent::SetPower { section_id, power } => {
                self.section_states.entry(section_id).or_default().power = power;
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
            }
            UiSectionEvent::HardwareSectionEvent(hw_section_event) => {
                match hw_section_event.event_type {
                    SectionEventType::Freed => {
                        // only used in the ui
                        self.section_states
                            .entry(hw_section_event.section_id.into())
                            .or_default()
                            .occupant
                            .was_freed();
                    }
                    SectionEventType::Occupied => {}
                }
            }
        }
    }

    fn handle_switch_event(&mut self, switch_event: UiSwitchEvent) {
        match switch_event {
            UiSwitchEvent::SetState { id, state } => {
                self.switch_states.insert(id, state);
            }
        }
    }

    fn handle_train_event(&mut self, train_event: UiTrainEvent) {
        match train_event {
            UiTrainEvent::EnteredSection {
                train_id,
                section_id,
            } => {
                self.trains.get_mut(&train_id).unwrap().current_section = Some(section_id);
            }
            UiTrainEvent::SpeedChanged { train_id, speed } => {
                self.trains.get_mut(&train_id).unwrap().speed = speed;
            }
            UiTrainEvent::StateChanged { train_id, state } => {
                self.trains.get_mut(&train_id).unwrap().state = state;
            }
            _ => todo!(),
        }
    }

    fn handle_event(&mut self, event: UiEvent, cx: &mut Context<Self>) {
        // add to logs
        self.logs.push(ControllerUiLog::ui_event(&event));
        cx.notify();

        // update own state based on event
        match event.clone() {
            UiEvent::UiSectionEvent(section_event) => self.handle_section_event(section_event),
            UiEvent::UiSwitchEvent(switch_event) => self.handle_switch_event(switch_event),
            UiEvent::UiTrainEvent(train_event) => self.handle_train_event(train_event),
            UiEvent::HardwareCommand(_) => {}
        }

        // and then also emit the event to any listeners
        cx.emit(event);
    }
}

pub struct ControllerUiWrapper {
    // has to be Option because we need to `.take()` it out when starting
    controller: Option<Controller>,
    controller_state: Entity<ControllerUiWrapperState>,

    layout: Option<ResolvedLayout>,

    command_tx: crossbeam::channel::Sender<UiCommand>,

    _task: Option<Task<()>>,
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
            layout: None,
            _task: Some(_task),
        }
    }

    pub fn with_layout(mut self, layout: ResolvedLayout) -> Self {
        self.layout = Some(layout);
        self
    }

    pub fn can_start(cx: &App) -> bool {
        cx.global::<Self>().controller.is_some()
    }

    pub fn start(cx: &mut App) {
        cx.update_global(|this: &mut Self, _| {
            let controller = this
                .controller
                .take()
                .expect("controller should be present. Please only call start() once");

            std::thread::spawn(move || controller.start());
        });
    }

    pub fn exec(command: impl Into<UiCommand>, cx: &App) {
        let command = command.into();
        let _ = cx.global::<Self>().command_tx.send(command);
    }

    pub fn state(cx: &App) -> &Entity<ControllerUiWrapperState> {
        &cx.global::<Self>().controller_state
    }

    pub fn layout(cx: &App) -> Option<&ResolvedLayout> {
        cx.global::<Self>().layout.as_ref()
    }
}

impl Global for ControllerUiWrapper {}
impl EventEmitter<UiEvent> for ControllerUiWrapper {}
