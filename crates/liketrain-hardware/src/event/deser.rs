use crate::{
    deser::{Deser, DeserError, DeserPayloadReader, DeserPayloadWriter},
    deser_variant,
    event::{HardwareEvent, HardwareSwitchId, HardwareSwitchState, SectionEvent},
};

deser_variant! {
    HardwareEventType {
        Pong = 0x1,
        SectionEvent = 0x2,
        SwitchStateChange = 0x3,

        Slaves = 0x10
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct HardwareEventSwitchStateChange {
    pub switch_id: HardwareSwitchId,
    pub state: HardwareSwitchState,
}

impl Deser for HardwareEvent {
    type Variant = HardwareEventType;

    type Error = ();

    fn payload_size(variant: Self::Variant) -> crate::deser::DeserSize {
        match variant {
            HardwareEventType::SectionEvent => core::mem::size_of::<SectionEvent>().into(),
            HardwareEventType::Pong => 8.into(),
            HardwareEventType::SwitchStateChange => {
                core::mem::size_of::<HardwareEventSwitchStateChange>().into()
            }
            HardwareEventType::Slaves => 4.into(),
        }
    }

    fn variant(&self) -> Self::Variant {
        match self {
            Self::SectionEvent { .. } => HardwareEventType::SectionEvent,
            Self::Pong { .. } => HardwareEventType::Pong,
            Self::SwitchStateChanged { .. } => HardwareEventType::SwitchStateChange,
            Self::Slaves { .. } => HardwareEventType::Slaves,
        }
    }

    fn deser_deserialize(
        variant: Self::Variant,
        _payload_size: u32,
        mut payload: DeserPayloadReader,
    ) -> Result<Self, DeserError<Self::Error>> {
        match variant {
            HardwareEventType::Pong => {
                let slave_id = payload.parse_u32()?;
                let seq = payload.parse_u32()?;
                Ok(Self::Pong { slave_id, seq })
            }
            HardwareEventType::Slaves => {
                let n_slaves = payload.parse_u32()?;
                Ok(Self::Slaves { n_slaves })
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
        }
    }

    fn deser_serialize(
        &self,
        mut buffer: DeserPayloadWriter,
    ) -> Result<(), DeserError<Self::Error>> {
        match self {
            &Self::Pong { slave_id, seq } => {
                buffer.write_u32(slave_id)?;
                buffer.write_u32(seq)?;
                Ok(())
            }
            &Self::Slaves { n_slaves } => {
                buffer.write_u32(n_slaves)?;
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
        }
    }
}
