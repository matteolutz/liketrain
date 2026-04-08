use std::{collections::HashMap, time::Duration};

use crossbeam::{channel::tick, select};
use liketrain_hardware::{command::HardwareCommand, event::HardwareEvent};

mod train;
pub use train::*;

use crate::{SectionId, SwitchId, SwitchState, comm::ControllerHardwareCommunication};

#[derive(Default)]
pub struct SimHardwareCommunication {
    trains: Vec<SimTrain>,
}

impl SimHardwareCommunication {
    pub fn new(trains: impl IntoIterator<Item = SimTrain>) -> Self {
        Self {
            trains: trains.into_iter().collect(),
        }
    }
}

impl ControllerHardwareCommunication for SimHardwareCommunication {
    fn start(
        &self,
        channels: super::ControllerHardwareCommunicationChannels,
    ) -> Result<(), crate::ControllerError> {
        let mut trains = self.trains.clone();

        std::thread::spawn(move || {
            let mut section_states = HashMap::new();
            let mut switch_states = HashMap::new();

            let ticker = tick(Duration::from_millis(10));

            let mut events = Vec::new();

            loop {
                select! {
                    recv(channels.command_rx) -> command => {
                        if let Ok(command) = command {
                            match command {
                                HardwareCommand::ResetAll => {
                                    section_states.clear();
                                    switch_states.clear();
                                }
                                HardwareCommand::GetSlaves => {
                                    events.push(HardwareEvent::Slaves { n_slaves: 0 });
                                }
                                HardwareCommand::Ping { slave_id, seq } => if slave_id == 0 {
                                    events.push(HardwareEvent::Pong { slave_id, seq });
                                },
                                HardwareCommand::SetSectionPower { section_id, power } => {
                                    section_states.insert(SectionId::from(section_id), power);
                                    events.push(HardwareEvent::SectionPowerChanged { section_id, power });
                                }
                                HardwareCommand::SetSwitchState { switch_id, state } => {
                                    switch_states.insert(SwitchId::from_hardware_id(&switch_id), SwitchState::from(state));
                                    events.push(HardwareEvent::SwitchStateChanged { switch_id, state });
                                }
                            }
                        }
                    }
                    recv(ticker) -> _ => {
                        for train in trains.iter_mut() {
                            train.update(&section_states, &switch_states, &mut events);
                        }

                       for event in events.drain(..)  {
                           let _ = channels.event_tx.send(event);
                       }
                    }
                }
            }
        });

        Ok(())
    }
}
