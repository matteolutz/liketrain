use std::{
    collections::{HashMap, VecDeque},
    time::Duration,
};

use crate::{
    SectionId, SectionTransitionSwitchChange, SwitchId, SwitchState, Track, Train, TrainId,
    controller::comm::{ControllerHardwareCommunication, ControllerHardwareCommunicationChannels},
    ui::{UiCommand, UiEvent, UiSectionEvent, UiSwitchEvent, UiTrainEvent},
};

mod state;
use itertools::Itertools;
use liketrain_hardware::{
    command::HardwareCommand,
    event::{HardwareEvent, HardwareSectionPower, HardwareSwitchId, SectionEventType},
};
pub use state::*;

pub mod comm;

mod error;
pub use error::*;

mod scheduler;
pub use scheduler::*;

mod event;
pub use event::*;

pub mod ui;

pub struct ControllerConfig {
    pub track: Track,
    pub trains: HashMap<TrainId, Train>,
}

#[derive(Copy, Clone)]
struct EventExecutionContext<'a> {
    command_tx: &'a crossbeam::channel::Sender<HardwareCommand>,
    event_rx: &'a crossbeam::channel::Receiver<HardwareEvent>,
}

impl<'a> EventExecutionContext<'a> {
    pub fn exec(&self, command: impl Into<HardwareCommand>) -> Result<(), ControllerError> {
        let command = command.into();
        log::debug!("sending hw command: {:?}", command);

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

    ui_event_tx: std::sync::mpsc::Sender<UiEvent>,
    ui_command_rx: crossbeam::channel::Receiver<UiCommand>,
    // TODO: ui_command_rx
}

impl Controller {
    pub fn new(
        config: ControllerConfig,
        hardware_comm: impl ControllerHardwareCommunication + 'static,
        ui_event_tx: std::sync::mpsc::Sender<UiEvent>,
        ui_command_rx: crossbeam::channel::Receiver<UiCommand>,
    ) -> Self {
        Self {
            section_states: config
                .track
                .sections()
                .map(|(section_id, _)| (section_id, SectionState::default()))
                .collect(),
            switch_states: config
                .track
                .switches()
                .map(|(switch_id, _)| (switch_id.clone(), SwitchState::default()))
                .collect(),
            track: config.track,
            trains: config.trains,
            scheduler: Scheduler::default(),
            section_queues: HashMap::new(),
            section_reservations: HashMap::new(),
            hardware_comm: Box::new(hardware_comm),
            ui_event_tx,
            ui_command_rx,
        }
    }

    pub fn trains(&self) -> impl Iterator<Item = (TrainId, &Train)> {
        self.trains.iter().map(|(id, train)| (*id, train))
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

    pub fn track(&self) -> &Track {
        &self.track
    }

    pub fn switch_states(&self) -> impl Iterator<Item = (SwitchId, SwitchState)> {
        self.switch_states
            .iter()
            .map(|(id, state)| (id.clone(), *state))
    }

    pub fn section_states(&self) -> impl Iterator<Item = (SectionId, &SectionState)> {
        self.section_states.iter().map(|(id, state)| (*id, state))
    }

    pub fn section_reservation(&self, section_id: SectionId) -> Option<TrainId> {
        self.section_reservations.get(&section_id).copied()
    }

    pub fn section_queue(&self, section_id: SectionId) -> Option<&VecDeque<TrainId>> {
        self.section_queues.get(&section_id)
    }
}

impl Controller {
    fn emit_ui(&self, event: impl Into<UiEvent>) {
        let _ = self.ui_event_tx.send(event.into());
    }
}

impl Controller {
    fn set_switch_state(&mut self, switch_id: SwitchId, state: impl Into<SwitchState>) {
        let state = state.into();
        self.switch_states.insert(switch_id.clone(), state);

        self.emit_ui(UiSwitchEvent::SetState {
            id: switch_id,
            state,
        });
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
        self.emit_ui(UiSectionEvent::Reserved {
            section_id,
            train_id: Some(train_id),
        });

        true
    }

    fn release_reservation(&mut self, section_id: SectionId, train_id: TrainId) {
        if self.section_reservations.get(&section_id) == Some(&train_id) {
            self.section_reservations.remove(&section_id);
            self.emit_ui(UiSectionEvent::Reserved {
                section_id,
                train_id: None,
            });
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

            if let Some(inbound_train_curent_section) = inbound_train.get_current_section() {
                self.scheduler
                    .schedule_now(ScheduledEvent::TrainLeftSection {
                        train_id: **inbound_train_id,
                        section_id: inbound_train_curent_section,
                    });
            }

            // it's just one train, so this must be the train that just entered this section
            inbound_train.entered_section(section_id);
            self.scheduler
                .schedule_now(ScheduledEvent::TrainEnteredSection {
                    train_id: **inbound_train_id,
                    section_id,
                });

            state.occupied = Some(**inbound_train_id);
        } else {
            // Don't emit a TrainLeftSection event right here.
            // If we stop a train because it has to wait, the hardware
            // will also emit a SectionFreed event
        }
    }

    fn set_section_power(
        &mut self,
        section_id: SectionId,
        power: HardwareSectionPower,
        _ctx: EventExecutionContext,
    ) {
        let state = self.section_states.entry(section_id).or_default();
        state.power = power;

        self.emit_ui(UiSectionEvent::SetPower { section_id, power });
    }
}

impl Controller {
    fn handle_hardware_event(
        &mut self,
        event: HardwareEvent,
        ctx: EventExecutionContext,
    ) -> Result<(), ControllerError> {
        match event {
            HardwareEvent::SectionEvent(section_event) => {
                // also emit this to the UI
                self.emit_ui(UiSectionEvent::HardwareSectionEvent(section_event));

                match section_event.event_type {
                    SectionEventType::Occupied => {
                        self.set_section_occupied(section_event.section_id.into(), true, ctx)
                    }
                    SectionEventType::Freed => {
                        self.set_section_occupied(section_event.section_id.into(), false, ctx)
                    }
                }
            }
            HardwareEvent::SectionPowerChanged { section_id, power } => {
                self.set_section_power(section_id.into(), power, ctx)
            }
            HardwareEvent::SwitchStateChanged { switch_id, state } => {
                let switch_id = SwitchId::from_hardware_id(&switch_id);

                self.set_switch_state(switch_id, state);
            }
            HardwareEvent::Pong { slave_id, seq } => {
                println!("received pong from slave {} with seq {}", slave_id, seq)
            }
            HardwareEvent::Slaves { n_slaves } => {
                println!("received slaves event with n_slaves: {}", n_slaves);
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
                self.emit_ui(UiSectionEvent::Occupied {
                    section_id: current_section_id,
                    train_id: Some(train_id),
                });
                self.emit_ui(UiTrainEvent::EnteredSection {
                    train_id,
                    section_id: current_section_id,
                });

                let train = self.train(train_id)?;

                if let Some(transition) = train.get_transition_to_next_section().cloned() {
                    let next_section = transition.destination();

                    log::debug!("next section is: {}", next_section);

                    // don't just check, if the section is occupied, but also if there are other trains inbound
                    // for the next section. if there are other trains inbound, we need to resolve the conflict.
                    // Probably have some sort of waiting queue for each section, and if there are other trains inbound
                    // just append this train to the queue.

                    if self.is_section_available(next_section, train_id) {
                        self.try_reserve_section(next_section, train_id);

                        // set required switches to the next section first
                        for SectionTransitionSwitchChange {
                            switch_id,
                            required_state,
                            ..
                        } in transition.required_switch_changes()
                        {
                            let hw_switch_id: HardwareSwitchId = switch_id.try_into().unwrap();
                            ctx.exec(HardwareCommand::SetSwitchState {
                                switch_id: hw_switch_id,
                                state: required_state.into(),
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
                        self.emit_ui(UiSectionEvent::QueueEnqueued {
                            section_id: next_section,
                            train_id,
                        });
                    }
                }
            }
            ScheduledEvent::TrainLeftSection {
                train_id,
                section_id,
            } => {
                self.emit_ui(UiSectionEvent::Occupied {
                    section_id,
                    train_id: None,
                });
                self.release_reservation(section_id, train_id);

                while let Some(waiting_train_id) = self
                    .section_queues
                    .get_mut(&section_id)
                    .and_then(|queue| queue.pop_front())
                {
                    self.emit_ui(UiSectionEvent::QueueDequeued {
                        section_id,
                        train_id: waiting_train_id,
                    });

                    // this train was on the queue for this section id
                    // this means, either the section was occupied before
                    // or there was another train inbound
                    let train = self.trains.get(&waiting_train_id).unwrap();

                    if train
                        .get_next_section()
                        .is_none_or(|next_section| next_section != section_id)
                    {
                        // this train has changed its mind? it doesn't want to go to this section anymore
                        // check if there's another train waiting on the queue
                        continue;
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
                    for SectionTransitionSwitchChange {
                        switch_id,
                        required_state,
                        ..
                    } in transition.required_switch_changes()
                    {
                        let hw_switch_id: HardwareSwitchId = switch_id.try_into().unwrap();
                        ctx.exec(HardwareCommand::SetSwitchState {
                            switch_id: hw_switch_id,
                            state: required_state.into(),
                        })?;
                    }

                    // power the next section
                    ctx.exec(HardwareCommand::SetSectionPower {
                        section_id: section_id.as_u32(),
                        power: HardwareSectionPower::Full,
                    })?;

                    return Ok(());
                }

                // there are now waiting trains, unpower this section
                ctx.exec(HardwareCommand::SetSectionPower {
                    section_id: section_id.as_u32(),
                    power: HardwareSectionPower::Off,
                })?;
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
        log::debug!("handling event: {:?}", event);

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

    fn handle_ui_command(
        &mut self,
        command: UiCommand,
        ctx: EventExecutionContext,
    ) -> Result<(), ControllerError> {
        match command {
            UiCommand::SetSectionPower { section_id, power } => {
                ctx.exec(HardwareCommand::SetSectionPower {
                    section_id: section_id.as_u32(),
                    power,
                })?;
            }
            UiCommand::SetSwitchState { switch_id, state } => {
                ctx.exec(HardwareCommand::SetSwitchState {
                    switch_id: switch_id.try_into().unwrap(),
                    state: state.into(),
                })?;
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

    fn init(&mut self, ctx: EventExecutionContext) -> Result<(), ControllerError> {
        log::debug!("getting slaves from master");
        ctx.exec(HardwareCommand::GetSlaves)?;

        let event = ctx.event_rx.recv()?;
        let HardwareEvent::Slaves { n_slaves } = event else {
            log::debug!("expected HardwareEvent::Slaves, got {:?}", event);
            return Err(ControllerError::ExpectedHardwareEvent(
                HardwareEvent::Slaves { n_slaves: 0 },
            ));
        };

        log::debug!("we have {} slaves", n_slaves);

        let ping_seq = 1337;

        // send pings
        for device_id in 0..=n_slaves {
            log::debug!("sending ping to device {}", device_id);

            ctx.exec(HardwareCommand::Ping {
                slave_id: device_id,
                seq: ping_seq,
            })?;

            if let HardwareEvent::Pong { slave_id, seq } = ctx.event_rx.recv()?
                && slave_id == device_id
                && seq == ping_seq
            {
                log::debug!("received pong from device {}", device_id);
                // okay, we received the correct pong
            } else {
                return Err(ControllerError::ExpectedHardwareEvent(
                    HardwareEvent::Pong {
                        slave_id: device_id,
                        seq: ping_seq,
                    },
                ));
            };
        }

        // reset everything
        ctx.exec(HardwareCommand::ResetAll)?;

        // initialize all the trains
        for (_, train) in &self.trains {
            let Some(initial_section) = train.get_initial_section() else {
                continue;
            };

            // power on the current section
            ctx.exec(HardwareCommand::SetSectionPower {
                section_id: initial_section.as_u32(),
                power: HardwareSectionPower::Full,
            })?;

            // powering up the initial section will cause the train to trigger the train detection sensor
            // and cause a SectionOccupied event. Because the trains current section will be set to None,
            // the `set_section_occupied` method will correctly identify this train as inbound to
            // the initial section.

            /*
            // this will set the next switches and power to the next section
            self.scheduler
                .schedule_now(ScheduledEvent::TrainEnteredSection {
                    train_id: id,
                    section_id: current_section,
                });*/
        }

        Ok(())
    }

    #[allow(unused)]
    fn test_switch(switch_id: SwitchId, ctx: EventExecutionContext) -> Result<(), ControllerError> {
        log::debug!("Testing switch: {}", switch_id);

        ctx.exec(HardwareCommand::SetSwitchState {
            switch_id: HardwareSwitchId::try_from(switch_id.clone()).unwrap(),
            state: liketrain_hardware::event::HardwareSwitchState::Right,
        })?;
        std::thread::sleep(Duration::from_secs(2));
        ctx.exec(HardwareCommand::SetSwitchState {
            switch_id: HardwareSwitchId::try_from(switch_id.clone()).unwrap(),
            state: liketrain_hardware::event::HardwareSwitchState::Left,
        })?;

        std::thread::sleep(Duration::from_secs(2));

        Ok(())
    }

    #[allow(unused)]
    fn test_switches(&self, ctx: EventExecutionContext) -> Result<(), ControllerError> {
        for (switch_id, _) in self
            .track
            .switches()
            .sorted_by_key(|(switch_id, _)| *switch_id)
        {
            Self::test_switch(switch_id.clone(), ctx)?;
        }

        Ok(())
    }

    #[allow(unused)]
    fn test_section(
        section_id: SectionId,
        ctx: EventExecutionContext,
    ) -> Result<(), ControllerError> {
        log::debug!("Testing section: {}", section_id);

        ctx.exec(HardwareCommand::SetSectionPower {
            section_id: section_id.as_u32(),
            power: HardwareSectionPower::Quarter,
        })?;
        std::thread::sleep(Duration::from_secs(1));
        ctx.exec(HardwareCommand::SetSectionPower {
            section_id: section_id.as_u32(),
            power: HardwareSectionPower::Half,
        })?;
        std::thread::sleep(Duration::from_secs(1));
        ctx.exec(HardwareCommand::SetSectionPower {
            section_id: section_id.as_u32(),
            power: HardwareSectionPower::ThreeQuarters,
        })?;
        std::thread::sleep(Duration::from_secs(1));
        ctx.exec(HardwareCommand::SetSectionPower {
            section_id: section_id.as_u32(),
            power: HardwareSectionPower::Full,
        })?;
        std::thread::sleep(Duration::from_secs(1));
        ctx.exec(HardwareCommand::SetSectionPower {
            section_id: section_id.as_u32(),
            power: HardwareSectionPower::Off,
        })?;

        std::thread::sleep(Duration::from_secs(5));

        Ok(())
    }

    #[allow(unused)]
    fn test_sections(&self, ctx: EventExecutionContext) -> Result<(), ControllerError> {
        for (section_id, _) in self
            .track
            .sections()
            .sorted_by_key(|(section_id, _)| section_id.as_u32())
        {
            Self::test_section(section_id, ctx)?;
        }

        Ok(())
    }

    pub fn start(mut self) -> Result<(), ControllerError> {
        let (command_tx, command_rx) = crossbeam::channel::unbounded();
        let (event_tx, event_rx) = crossbeam::channel::unbounded();

        // start the hardware communication
        self.hardware_comm
            .start(ControllerHardwareCommunicationChannels {
                event_tx,
                command_rx,
            })?;

        std::thread::sleep(Duration::from_secs(5));

        let ctx = EventExecutionContext {
            command_tx: &command_tx,
            event_rx: &event_rx,
        };

        // initialize the controller
        self.init(ctx)?;

        // test the sections
        // self.test_sections(ctx)?;
        // Self::test_section(26_usize.into(), ctx)?;

        // test the switches
        // self.test_switches(ctx)?;

        /*for _ in 0..100 {
            Self::test_switch("B".into(), ctx)?;
        }*/

        // main loop
        loop {
            // if we don't have any scheduled events,
            // wait 1 second before running again
            let event_timeout = self
                .scheduler
                .next_event_duration()
                .unwrap_or(Duration::from_millis(500));

            crossbeam::select! {
                recv(event_rx) -> event => {
                    if let Ok(event) = event {
                        self.handle_event(event, ctx)?;
                    }
                }
                recv(self.ui_command_rx) -> command => {
                    if let Ok(command) = command {
                        self.handle_ui_command(command, ctx)?;
                    }
                }
                default(event_timeout)  => {
                    self.resolve_pending_events(ctx)?;
                }
            }
        }
    }
}
