use std::time::Duration;

use crossbeam::{channel::tick, select};
use liketrain_hardware::{
    event::HardwareEvent,
    serial::{DeserSerialExt, Serial},
};

use crate::{controller::comm::ControllerHardwareCommunication, serial::SerialportSerialInterface};

pub struct SerialControllerHardwareCommunication {
    port_name: String,
    baud_rate: u32,
}

impl SerialControllerHardwareCommunication {
    pub fn new(port_name: &str, baud_rate: u32) -> Self {
        Self {
            port_name: port_name.to_string(),
            baud_rate,
        }
    }
}

impl ControllerHardwareCommunication for SerialControllerHardwareCommunication {
    fn start(
        &self,
        channels: super::ControllerHardwareCommunicationChannels,
    ) -> Result<(), crate::ControllerError> {
        let port = serialport::new(&self.port_name, self.baud_rate)
            .timeout(Duration::from_millis(20))
            .open()?;

        std::thread::spawn(move || {
            let mut interface: SerialportSerialInterface = port.into();
            let mut serial = Serial::new(&mut interface);

            let ticker = tick(Duration::from_millis(50));

            loop {
                select! {
                    recv(channels.command_rx) -> command => {
                        if let Ok(command) = command {
                            serial.write(&command).unwrap();
                        }
                    }
                    recv(ticker) -> _ => {
                        serial.update().unwrap();

                        while let Some(event) = serial.read::<HardwareEvent>().unwrap() {
                            match event {
                                   HardwareEvent::DebugMessage { message } => {
                                       log::info!("Debug message: {}", message);
                                   },
                                   event => {
                                       let _ = channels.event_tx.send(event);
                                   }
                               }

                        }
                    }
                }
            }
        });

        Ok(())
    }
}
