use std::{
    ops::{Deref, DerefMut},
    sync::mpsc,
    time::Duration,
};

use gpui::{App, AppContext, Entity, EventEmitter, Global, Task};
use liketrain_core::{
    Controller, ControllerConfig,
    comm::ControllerHardwareCommunication,
    ui::{UiCommand, UiEvent},
};

struct ControllerEventEmitter {}
impl EventEmitter<UiEvent> for ControllerEventEmitter {}

pub struct ControllerUiWrapper {
    controller: Controller,

    event_emitter: Entity<ControllerEventEmitter>,

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

        let event_emitter = cx.new(|_| ControllerEventEmitter {});

        let _task = cx.spawn({
            let event_emitter = event_emitter.clone();
            async move |cx| {
                loop {
                    cx.background_executor()
                        .timer(Duration::from_millis(16))
                        .await;

                    for event in event_rx.try_iter() {
                        event_emitter.update(cx, |_, cx| cx.emit(event));
                    }
                }
            }
        });

        Self {
            controller,
            command_tx,
            event_emitter,
            task: None,
        }
    }

    pub fn send(&self, command: impl Into<UiCommand>) {
        let command = command.into();
        self.command_tx.send(command);
    }

    pub fn event_emitter(&self) -> &Entity<ControllerEventEmitter> {
        &self.event_emitter
    }
}

impl Global for ControllerUiWrapper {}
impl EventEmitter<UiEvent> for ControllerUiWrapper {}

impl Deref for ControllerUiWrapper {
    type Target = Controller;

    fn deref(&self) -> &Self::Target {
        &self.controller
    }
}

impl DerefMut for ControllerUiWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.controller
    }
}
