#[cfg(feature = "avr")]
pub use alloc::string::String;

use crate::event::HardwareEvent;

pub mod deser;

#[derive(Debug, Clone)]
pub enum HardwareResponse {
    Ack,

    DebugMessage { message: String },

    Event(HardwareEvent),
}
