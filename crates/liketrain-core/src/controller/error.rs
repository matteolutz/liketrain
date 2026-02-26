use crossbeam::channel::SendError;
use liketrain_hardware::command::HardwareCommand;
use thiserror::Error;

use crate::TrainId;

#[derive(Debug, Error)]
pub enum ControllerError {
    #[error("Command send error: {0}")]
    CommandSendError(#[from] SendError<HardwareCommand>),

    #[error("Train not found: {0}")]
    TrainNotFound(TrainId),
}
