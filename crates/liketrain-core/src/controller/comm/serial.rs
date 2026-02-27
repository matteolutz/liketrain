use std::{io, time::Duration};

use crossbeam::{channel::tick, select};
use liketrain_hardware::event::HardwareEvent;

use crate::{controller::comm::ControllerHardwareCommunication, serial::SerialExt};

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
        let mut port = serialport::new(&self.port_name, self.baud_rate)
            .timeout(Duration::from_millis(20))
            .open()?;

        std::thread::spawn(move || {
            let ticker = tick(Duration::from_millis(10));

            let mut stream_buffer = Vec::new();
            let mut read_buf = [0_u8; 64];

            loop {
                select! {
                    recv(channels.command_rx) -> command => {
                        if let Ok(command) = command {
                            let _ = port.write_command(command).unwrap();
                        }
                    }
                    recv(ticker) -> _ => {
                        match port.read(&mut read_buf) {
                            Ok(n) if n > 0 => {
                                stream_buffer.extend_from_slice(&read_buf[..n]);

                                while let Some(event) = port.try_read_event_from_stream(&mut stream_buffer).unwrap() {
                                    match &event {
                                        &HardwareEvent::DebugMessage { len } => {
                                            let message = port.read_debug_message(len as usize).unwrap();
                                            log::info!("Debug message: {}", message);
                                        },
                                        _=> {}
                                    }
                                    let _ = channels.event_tx.send(event);
                                }
                            }
                            Err(err) if err.kind() == io::ErrorKind::TimedOut => {},
                            Err(err) => log::warn!("Failed to read from serial port: {}", err),
                            _ => {}
                        }
                    }
                }
            }
        });

        Ok(())
    }
}
