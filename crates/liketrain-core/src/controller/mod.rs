use std::collections::HashMap;

use crate::{
    SectionId, SwitchId, SwitchState, Track, Train, TrainId,
    controller::comm::{ControllerHardwareCommunication, ControllerHardwareCommunicationChannels},
};

mod state;
use liketrain_hardware::{
    command::HardwareCommand,
    event::{HardwareEvent, SectionEventType},
};
pub use state::*;

mod comm;

mod error;
pub use error::*;

mod scheduler;
pub use scheduler::*;

mod event;
pub use event::*;

pub struct ControllerConfig {
    pub track: Track,
    pub trains: HashMap<TrainId, Train>,
}

#[derive(Copy, Clone)]
struct EventExecutionContext<'a> {
    command_tx: &'a crossbeam::channel::Sender<HardwareCommand>,
}

impl<'a> EventExecutionContext<'a> {
    pub fn exec(&self, command: impl Into<HardwareCommand>) -> Result<(), ControllerError> {
        let command = command.into();
        self.command_tx.send(command)?;
        Ok(())
    }
}

pub struct Controller {
    track: Track,
    trains: HashMap<TrainId, Train>,

    section_states: HashMap<SectionId, SectionState>,
    switch_states: HashMap<SwitchId, SwitchState>,

    scheduler: Scheduler,

    hardware_comm: Box<dyn ControllerHardwareCommunication>,
}

impl Controller {
    pub fn new(
        config: ControllerConfig,
        hardware_comm: impl ControllerHardwareCommunication + 'static,
    ) -> Self {
        Self {
            track: config.track,
            trains: config.trains,
            section_states: HashMap::new(),
            switch_states: HashMap::new(),
            scheduler: Scheduler::default(),
            hardware_comm: Box::new(hardware_comm),
        }
    }
}

impl Controller {
    fn set_switch_state(&mut self, switch_id: SwitchId, state: impl Into<SwitchState>) {
        self.switch_states.insert(switch_id, state.into());
    }

    fn set_section_occupied(
        &mut self,
        section_id: SectionId,
        occupied: bool,
        _ctx: EventExecutionContext,
    ) {
        let state = self.section_states.entry(section_id).or_default();
        let previous_occupied = state.occupied.take();

        if occupied {
            if previous_occupied.is_some() {
                // well, this shouldn't happen
                // TODO: how to handle this??
            }

            let mut inbound_trains = self
                .trains
                .iter_mut()
                .filter(|(_, train)| {
                    train
                        .get_next_section()
                        .is_some_and(|next_section| next_section == section_id)
                })
                .collect::<Vec<_>>();

            if inbound_trains.len() == 1 {
                let (id, train) = &mut inbound_trains[0];

                // it's just one train, so this must be the train that just entered this section
                train.entered_section(section_id);
                self.scheduler
                    .schedule_now(ScheduledEvent::TrainEnteredSection {
                        train_id: **id,
                        section_id,
                    });

                state.occupied = Some(**id);

                return;
            }

            // TODO: probably prevent collision
        } else {
            // if this section was occupied, send TrainLeftSection
            if let Some(previous_occupied) = previous_occupied {
                self.scheduler
                    .schedule_now(ScheduledEvent::TrainLeftSection {
                        train_id: previous_occupied,
                        section_id,
                    });
            }
        }
    }
}

impl Controller {
    fn handle_hardware_event(
        &mut self,
        event: HardwareEvent,
        ctx: EventExecutionContext,
    ) -> Result<(), ControllerError> {
        match event {
            HardwareEvent::SectionEvent(section_event) => match section_event.event_type {
                SectionEventType::Occupied => {
                    self.set_section_occupied(section_event.section_id.into(), true, ctx)
                }
                SectionEventType::Freed => {
                    self.set_section_occupied(section_event.section_id.into(), false, ctx)
                }
            },
            HardwareEvent::SwitchStateChanged { switch_id, state } => {
                let switch_id_str = str::from_utf8(&switch_id).unwrap();
                let switch_id: SwitchId = switch_id_str.into();

                self.set_switch_state(switch_id, state);
            }
            HardwareEvent::Pong(pong_id) => println!("received pong {}", pong_id),
        }
        Ok(())
    }

    fn handle_scheduled_event(
        &mut self,
        event: ScheduledEvent,
        _ctx: EventExecutionContext,
    ) -> Result<(), ControllerError> {
        match event {
            ScheduledEvent::TrainEnteredSection {
                train_id: _,
                section_id: _,
            } => {
                // TODO: if this trains transition into it's next section goes through switches, set them
                // and if the next section is occupied, stop the train
            }
            ScheduledEvent::TrainLeftSection {
                train_id: _,
                section_id: _,
            } => {
                // TODO: for now, i don't think, we need to do anything here
            }
        }

        Ok(())
    }

    fn handle_event(
        &mut self,
        event: impl Into<ControllerEvent>,
        ctx: EventExecutionContext,
    ) -> Result<(), ControllerError> {
        let event = event.into();

        match event {
            ControllerEvent::Scheduled(scheduled_event) => {
                self.handle_scheduled_event(scheduled_event, ctx)?
            }
            ControllerEvent::Hardware(hardware_event) => {
                self.handle_hardware_event(hardware_event, ctx)?
            }
        }

        Ok(())
    }

    fn resolve_pending_events(
        &mut self,
        ctx: EventExecutionContext,
    ) -> Result<(), ControllerError> {
        while let Some(event) = self.scheduler.next_event() {
            self.handle_event(event, ctx)?;
        }
        Ok(())
    }

    pub fn start(mut self) -> Result<(), ControllerError> {
        let (mut command_tx, command_rx) = crossbeam::channel::unbounded();
        let (event_tx, event_rx) = crossbeam::channel::unbounded();

        // start the hardware communication
        self.hardware_comm
            .start(ControllerHardwareCommunicationChannels {
                event_tx,
                command_rx,
            })?;

        // main loop
        loop {
            let ctx = EventExecutionContext {
                command_tx: &mut command_tx,
            };

            if let Some(event_timeout) = self.scheduler.next_event_duration() {
                crossbeam::select! {
                    recv(event_rx) -> event => {
                        if let Ok(event) = event {
                            self.handle_event(event, ctx)?;
                        }
                    }
                    default(event_timeout)  => {
                        self.resolve_pending_events(ctx)?;
                    }
                }
            } else if let Ok(event) = event_rx.recv() {
                self.handle_event(event, ctx)?;
            }
        }
    }
}
