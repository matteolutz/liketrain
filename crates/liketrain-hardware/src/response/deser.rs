#[cfg(feature = "avr")]
pub use alloc::string::String;
#[cfg(feature = "avr")]
use alloc::vec;

use crate::{
    deser::{Deser, DeserHelper},
    deser_variant,
    event::HardwareEvent,
    response::HardwareResponse,
};

deser_variant! {
    HardwareResponseType {
        Ack = 0x1,
        DebugMessage = 0x2,

        Event = 0x10,
    }
}

impl Deser for HardwareResponse {
    type Variant = HardwareResponseType;

    type Error = ();

    fn variant(&self) -> Self::Variant {
        match self {
            HardwareResponse::Ack => HardwareResponseType::Ack,
            HardwareResponse::DebugMessage { .. } => HardwareResponseType::DebugMessage,
            HardwareResponse::Event(_) => HardwareResponseType::Event,
        }
    }

    fn deser_deserialize(
        variant: Self::Variant,
        mut buffer: crate::deser::DeserPayloadReader,
    ) -> Result<Self, crate::deser::DeserError<Self::Error>> {
        match variant {
            HardwareResponseType::Ack => Ok(HardwareResponse::Ack),
            HardwareResponseType::DebugMessage => {
                let len = buffer.parse_u32()?;
                let mut str_buffer = vec![0_u8; len as usize];

                for byte in str_buffer.iter_mut() {
                    *byte = buffer.parse_u8()?;
                }

                let message = String::from_utf8(str_buffer).map_err(|_| ())?;
                Ok(HardwareResponse::DebugMessage { message })
            }
            HardwareResponseType::Event => {
                let event = HardwareEvent::deserialize(buffer.buffer())?;
                Ok(HardwareResponse::Event(event))
            }
        }
    }

    fn deser_serialize(
        &self,
        mut buffer: crate::deser::DeserPayloadWriter,
    ) -> Result<(), crate::deser::DeserError<Self::Error>> {
        match self {
            HardwareResponse::Ack => Ok(()),
            HardwareResponse::DebugMessage { message } => {
                let bytes = message.bytes();

                buffer.write_u32(bytes.len() as u32)?;
                for byte in bytes {
                    buffer.write_u8(byte)?;
                }

                Ok(())
            }
            HardwareResponse::Event(event) => {
                event.serialize_into(buffer.buffer())?;
                Ok(())
            }
        }
    }
}
