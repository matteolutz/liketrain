use liketrain_hardware::{command::HardwareCommand, event::HardwareEvent};

use crate::ControllerError;

pub struct ControllerHardwareCommunicationChannels {
    pub event_tx: crossbeam::channel::Sender<HardwareEvent>,
    pub command_rx: crossbeam::channel::Receiver<HardwareCommand>,
}

pub trait ControllerHardwareCommunication {
    /// Start the communication with the controller
    fn start(
        &self,
        channels: ControllerHardwareCommunicationChannels,
    ) -> Result<(), ControllerError>;
}
