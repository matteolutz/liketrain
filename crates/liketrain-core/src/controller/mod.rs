use std::collections::{HashMap, VecDeque};

use crate::{
    SectionId, SwitchId, SwitchState, Track, Train, TrainId,
    controller::comm::{ControllerHardwareCommunication, ControllerHardwareCommunicationChannels},
};

mod state;
use liketrain_hardware::{
    command::HardwareCommand,
    event::{HardwareEvent, HardwareSectionPower, HardwareSwitchId, SectionEventType},
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

    section_queues: HashMap<SectionId, VecDeque<TrainId>>,
    section_reservations: HashMap<SectionId, TrainId>,

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
            section_queues: HashMap::new(),
            section_reservations: HashMap::new(),
            hardware_comm: Box::new(hardware_comm),
        }
    }

    pub fn train(&self, train_id: TrainId) -> Result<&Train, ControllerError> {
        self.trains
            .get(&train_id)
            .ok_or(ControllerError::TrainNotFound(train_id))
    }

    pub fn train_mut(&mut self, train_id: TrainId) -> Result<&mut Train, ControllerError> {
        self.trains
            .get_mut(&train_id)
            .ok_or(ControllerError::TrainNotFound(train_id))
    }
}

impl Controller {
    fn set_switch_state(&mut self, switch_id: SwitchId, state: impl Into<SwitchState>) {
        self.switch_states.insert(switch_id, state.into());
    }

    fn try_reserve_section(&mut self, section_id: SectionId, train_id: TrainId) -> bool {
        if let Some(&existing_reservation) = self.section_reservations.get(&section_id) {
            if existing_reservation != train_id {
                return false;
            }

            // already reserved by the same train
            return true;
        }

        let section_state = self.section_states.entry(section_id).or_default();
        if let Some(occupant) = section_state.occupied {
            // can't reserve a section that is already occupied by another train
            if occupant != train_id {
                return false;
            }
        }

        self.section_reservations.insert(section_id, train_id);
        true
    }

    fn release_reservation(&mut self, section_id: SectionId, train_id: TrainId) {
        if self.section_reservations.get(&section_id) == Some(&train_id) {
            self.section_reservations.remove(&section_id);
        }
    }

    fn is_section_reserved_by_other(&self, section_id: SectionId, train_id: TrainId) -> bool {
        self.section_reservations
            .get(&section_id)
            .is_some_and(|&holder| holder != train_id)
    }

    fn is_section_available(&self, section_id: SectionId, for_train: TrainId) -> bool {
        !self.is_section_occupied(section_id)
            && !self.is_section_reserved_by_other(section_id, for_train)
    }

    fn is_section_occupied(&self, section_id: SectionId) -> bool {
        self.section_states
            .get(&section_id)
            .is_some_and(|state| state.occupied.is_some())
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

            if inbound_trains.is_empty() {
                // there are no inbound trains, which is weird
                // TODO: probably stop everything? something went wrong
                return;
            }

            if inbound_trains.len() > 1 {
                // multiple trains are inbound for the same section??? something went wrong
                // TODO: probably stop everything? something went wrong
                return;
            }

            let (inbound_train_id, inbound_train) = &mut inbound_trains[0];

            // it's just one train, so this must be the train that just entered this section
            inbound_train.entered_section(section_id);
            self.scheduler
                .schedule_now(ScheduledEvent::TrainEnteredSection {
                    train_id: **inbound_train_id,
                    section_id,
                });

            state.occupied = Some(**inbound_train_id);
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
                let switch_id = SwitchId::from_hardware_id(&switch_id);

                self.set_switch_state(switch_id, state);
            }
            HardwareEvent::Pong { slave_id, seq } => {
                println!("received pong from slave {} with seq {}", slave_id, seq)
            }
        }
        Ok(())
    }

    fn handle_scheduled_event(
        &mut self,
        event: ScheduledEvent,
        ctx: EventExecutionContext,
    ) -> Result<(), ControllerError> {
        match event {
            ScheduledEvent::TrainEnteredSection {
                train_id,
                section_id: current_section_id,
            } => {
                let train = self.train(train_id)?;

                if let Some(transition) = train.get_transition_to_next_section().cloned() {
                    let next_section = transition.destination();

                    // don't just check, if the section is occupied, but also if there are other trains inbound
                    // for the next section. if there are other trains inbound, we need to resolve the conflict.
                    // Probably have some sort of waiting queue for each section, and if there are other trains inbound
                    // just append this train to the queue.

                    if self.is_section_available(next_section, train_id) {
                        self.try_reserve_section(next_section, train_id);

                        // set required switches to the next section first
                        for (switch_id, state) in transition.required_switch_changes() {
                            let hw_switch_id: HardwareSwitchId = switch_id.try_into().unwrap();
                            ctx.exec(HardwareCommand::SetSwitchState {
                                switch_id: hw_switch_id,
                                state: state.into(),
                            })?;
                        }

                        // then set power to next section
                        ctx.exec(HardwareCommand::SetSectionPower {
                            section_id: next_section.as_u32(),
                            power: HardwareSectionPower::Full,
                        })?;
                    } else {
                        // stop the train
                        ctx.exec(HardwareCommand::SetSectionPower {
                            section_id: current_section_id.as_u32(),
                            power: HardwareSectionPower::Off,
                        })?;

                        // append it to the waiting trains
                        self.section_queues
                            .entry(next_section)
                            .or_default()
                            .push_back(train_id);
                    }
                }
            }
            ScheduledEvent::TrainLeftSection {
                train_id,
                section_id,
            } => {
                self.release_reservation(section_id, train_id);

                if let Some(waiting_train_id) = self
                    .section_queues
                    .get_mut(&section_id)
                    .and_then(|queue| queue.pop_front())
                {
                    // this train was on the queue for this section id
                    // this means, either the section was occupied before
                    // or there was another train inbound
                    let train = self.trains.get(&waiting_train_id).unwrap();

                    if train
                        .get_next_section()
                        .is_none_or(|next_section| next_section != section_id)
                    {
                        // this train has changed its mind? it doesn't want to go to this section anymore
                        return Ok(());
                    }

                    let current_section = train.get_current_section().unwrap();
                    let transition = train.get_transition_to_next_section().cloned().unwrap(); // safe to unwrap

                    self.try_reserve_section(section_id, train_id);

                    // restart the train (repower its current section)
                    ctx.exec(HardwareCommand::SetSectionPower {
                        section_id: current_section.as_u32(),
                        power: HardwareSectionPower::Full,
                    })?;

                    // set required switches to the next section
                    for (switch_id, state) in transition.required_switch_changes() {
                        let hw_switch_id: HardwareSwitchId = switch_id.try_into().unwrap();
                        ctx.exec(HardwareCommand::SetSwitchState {
                            switch_id: hw_switch_id,
                            state: state.into(),
                        })?;
                    }

                    // power the next section
                    ctx.exec(HardwareCommand::SetSectionPower {
                        section_id: section_id.as_u32(),
                        power: HardwareSectionPower::Full,
                    })?;
                } else {
                    // there are now waiting trains, unpower this section
                    ctx.exec(HardwareCommand::SetSectionPower {
                        section_id: section_id.as_u32(),
                        power: HardwareSectionPower::Off,
                    })?;
                }
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

        // TODO: maybe this recursion will become bad
        self.resolve_pending_events(ctx)?;

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
