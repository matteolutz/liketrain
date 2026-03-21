use std::{collections::HashMap, time::Duration};

use crossbeam::{channel::tick, select};
use liketrain_hardware::{command::HardwareCommand, event::HardwareSectionPower};

mod train;
pub use train::*;

use crate::{SectionId, comm::ControllerHardwareCommunication};

#[derive(Default)]
pub struct SimHardwareCommunication {
    trains: Vec<SimTrain>,
}

impl ControllerHardwareCommunication for SimHardwareCommunication {
    fn start(
        &self,
        channels: super::ControllerHardwareCommunicationChannels,
    ) -> Result<(), crate::ControllerError> {
        let mut trains = self.trains.clone();

        std::thread::spawn(move || {
            let mut section_states = HashMap::new();
            let mut switch_states: HashMap<SectionId, HardwareSectionPower> = HashMap::new();

            let ticker = tick(Duration::from_millis(10));

            let mut events = Vec::new();

            loop {
                select! {
                    recv(channels.command_rx) -> command => {
                        if let Ok(command) = command {
                            match command {
                                HardwareCommand::SetSectionPower { section_id, power } => {
                                    section_states.insert(SectionId::from(section_id), power);
                                }
                                _ => {}
                            }
                        }
                    }
                    recv(ticker) -> _ => {
                        for train in trains.iter_mut() {
                            train.update(&section_states, &mut events);
                        }

                       for event in events.drain(..)  {
                           let _ = channels.event_tx.send(event);
                       }
                    }
                }
            }
        });

        todo!()
    }
}
