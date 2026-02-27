use crossbeam::channel::{RecvError, SendError};
use liketrain_hardware::{command::HardwareCommand, event::HardwareEvent};
use thiserror::Error;

use crate::TrainId;

#[derive(Debug, Error)]
pub enum ControllerError {
    #[error("Command send error: {0}")]
    CommandSendError(#[from] SendError<HardwareCommand>),

    #[error("Train not found: {0}")]
    TrainNotFound(TrainId),

    #[error("Crossbeam receive error: {0}")]
    CrossbeamRecvError(#[from] RecvError),

    #[error("Expected hardware event: {0:?}")]
    ExpectedHardwareEvent(HardwareEvent),

    #[error("Serial communication error: {0}")]
    Serial(#[from] serialport::Error),
}
