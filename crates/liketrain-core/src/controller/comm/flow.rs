use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use liketrain_hardware::command::HardwareCommand;

pub struct SerialFlowController {
    command_buffer: VecDeque<HardwareCommand>,
    awaiting_ack: Option<Instant>,
}

impl SerialFlowController {
    const ACK_TIMEOUT: Duration = Duration::from_millis(500);

    pub fn new() -> Self {
        SerialFlowController {
            command_buffer: VecDeque::new(),
            awaiting_ack: None,
        }
    }

    pub fn push_command(&mut self, command: HardwareCommand) {
        self.command_buffer.push_back(command);
    }

    pub fn ack_received(&mut self) {
        self.awaiting_ack = None;
    }

    pub fn update(&mut self) -> Option<HardwareCommand> {
        if let Some(awaiting_ack) = self.awaiting_ack.as_ref() {
            if *awaiting_ack > Instant::now() {
                return None;
            }

            self.awaiting_ack = None;
        }

        if let Some(command) = self.command_buffer.pop_front() {
            self.awaiting_ack = Some(Instant::now() + Self::ACK_TIMEOUT);
            return Some(command);
        }

        None
    }
}
