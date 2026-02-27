#[cfg(feature = "avr")]
use alloc::string::String;
#[cfg(feature = "avr")]
use alloc::vec::Vec;

use crate::{
    deser::{Deser, DeserError, DeserPayloadReader, DeserPayloadWriter},
    event::{
        HardwareEvent, HardwareSwitchId, HardwareSwitchState, SectionEvent,
        avr::{HardwareEventSwitchStateChange, HardwareEventType},
    },
};

impl Deser for HardwareEvent {
    type Variant = HardwareEventType;

    type Error = ();

    fn payload_size(variant: Self::Variant) -> crate::deser::DeserSize {
        match variant {
            HardwareEventType::DebugMessage => crate::deser::DeserSize::Variable,
            HardwareEventType::SectionEvent => core::mem::size_of::<SectionEvent>().into(),
            HardwareEventType::Pong => 8.into(),
            HardwareEventType::SwitchStateChange => {
                core::mem::size_of::<HardwareEventSwitchStateChange>().into()
            }
            HardwareEventType::Ack => 0.into(),
        }
    }

    fn variant(&self) -> Self::Variant {
        match self {
            Self::DebugMessage { .. } => HardwareEventType::DebugMessage,
            Self::SectionEvent { .. } => HardwareEventType::SectionEvent,
            Self::Pong { .. } => HardwareEventType::Pong,
            Self::SwitchStateChanged { .. } => HardwareEventType::SwitchStateChange,
            Self::Ack => HardwareEventType::Ack,
        }
    }

    fn deser_deserialize(
        variant: Self::Variant,
        payload_size: u32,
        mut payload: DeserPayloadReader,
    ) -> Result<Self, DeserError<Self::Error>> {
        match variant {
            HardwareEventType::Pong => {
                let slave_id = payload.parse_u32()?;
                let seq = payload.parse_u32()?;
                Ok(Self::Pong { slave_id, seq })
            }
            HardwareEventType::SectionEvent => {
                let section_event: SectionEvent = payload.parse()?;
                Ok(Self::SectionEvent(section_event))
            }
            HardwareEventType::SwitchStateChange => {
                let switch_id: HardwareSwitchId = payload.parse()?;
                let state: HardwareSwitchState = payload.parse()?;

                Ok(Self::SwitchStateChanged { switch_id, state })
            }
            HardwareEventType::DebugMessage => {
                let mut bytes = Vec::with_capacity(payload_size as usize);

                for _ in 0..payload_size {
                    bytes.push(payload.parse_u8()?);
                }

                let message = String::from_utf8(bytes).map_err(|_| ())?;

                Ok(HardwareEvent::DebugMessage { message })
            }
            HardwareEventType::Ack => Ok(HardwareEvent::Ack),
        }
    }

    fn deser_serialize(
        &self,
        mut buffer: DeserPayloadWriter,
    ) -> Result<(), DeserError<Self::Error>> {
        match self {
            Self::DebugMessage { message } => {
                let bytes = message.as_bytes();
                buffer.write_size(bytes.len() as u32)?;

                for &byte in bytes {
                    buffer.write_u8(byte)?;
                }

                Ok(())
            }
            &Self::Pong { slave_id, seq } => {
                buffer.write_u32(slave_id)?;
                buffer.write_u32(seq)?;
                Ok(())
            }
            Self::SectionEvent(event) => {
                buffer.write(event)?;
                Ok(())
            }
            Self::SwitchStateChanged { switch_id, state } => {
                buffer.write(switch_id)?;
                buffer.write(state)?;
                Ok(())
            }
            Self::Ack => Ok(()),
        }
    }
}
