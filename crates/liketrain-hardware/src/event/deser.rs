use crate::{
    deser::{Deser, DeserError, DeserPayloadReader, DeserPayloadWriter},
    deser_variant,
    event::{
        HardwareEvent, HardwareSectionPower, HardwareSwitchId, HardwareSwitchState, SectionEvent,
    },
};

deser_variant! {
    HardwareEventType {
        Pong = 0x1,
        SectionEvent = 0x2,
        SwitchStateChange = 0x3,
        SectionPowerChanged = 0x4,

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

    fn variant(&self) -> Self::Variant {
        match self {
            Self::SectionEvent { .. } => HardwareEventType::SectionEvent,
            Self::SectionPowerChanged { .. } => HardwareEventType::SectionPowerChanged,
            Self::Pong { .. } => HardwareEventType::Pong,
            Self::SwitchStateChanged { .. } => HardwareEventType::SwitchStateChange,
            Self::Slaves { .. } => HardwareEventType::Slaves,
        }
    }

    fn deser_deserialize(
        variant: Self::Variant,
        mut payload: DeserPayloadReader,
    ) -> Result<Self, DeserError<Self::Error>> {
        match variant {
            HardwareEventType::Pong => {
                let slave_id = payload.read_u32()?;
                let seq = payload.read_u32()?;
                Ok(Self::Pong { slave_id, seq })
            }
            HardwareEventType::Slaves => {
                let n_slaves = payload.read_u32()?;
                Ok(Self::Slaves { n_slaves })
            }
            HardwareEventType::SectionEvent => {
                let section_event: SectionEvent = payload.read()?;
                Ok(Self::SectionEvent(section_event))
            }
            HardwareEventType::SwitchStateChange => {
                let switch_id: HardwareSwitchId = payload.read()?;
                let state: HardwareSwitchState = payload.read()?;

                Ok(Self::SwitchStateChanged { switch_id, state })
            }
            HardwareEventType::SectionPowerChanged => {
                let section_id = payload.read_u32()?;
                let power: HardwareSectionPower = payload.read()?;

                Ok(Self::SectionPowerChanged { section_id, power })
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
            Self::SectionPowerChanged { section_id, power } => {
                buffer.write_u32(*section_id)?;
                buffer.write(power)?;
                Ok(())
            }
        }
    }
}
