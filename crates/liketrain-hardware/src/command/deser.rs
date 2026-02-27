use crate::{
    command::{HardwareCommand, avr::HardwareCommandType},
    deser::Deser,
    event::{HardwareSectionPower, HardwareSwitchId, HardwareSwitchState},
};

impl Deser for HardwareCommand {
    type Variant = HardwareCommandType;

    type Error = ();

    fn payload_size(variant: Self::Variant) -> crate::deser::DeserSize {
        match variant {
            HardwareCommandType::Ping => 8.into(),
            HardwareCommandType::ResetAll => 0.into(),
            HardwareCommandType::SetSectionPower => {
                (4 + core::mem::size_of::<HardwareSectionPower>()).into()
            }
            HardwareCommandType::SetSwitchState => (core::mem::size_of::<HardwareSwitchId>()
                + core::mem::size_of::<HardwareSwitchState>())
            .into(),
        }
    }

    fn variant(&self) -> Self::Variant {
        match self {
            Self::Ping { .. } => HardwareCommandType::Ping,
            Self::ResetAll => HardwareCommandType::ResetAll,
            Self::SetSectionPower { .. } => HardwareCommandType::SetSectionPower,
            Self::SetSwitchState { .. } => HardwareCommandType::SetSwitchState,
        }
    }

    fn deser_deserialize(
        variant: Self::Variant,
        _payload_size: u32,
        mut buffer: crate::deser::DeserPayloadReader,
    ) -> Result<Self, crate::deser::DeserError<Self::Error>> {
        match variant {
            HardwareCommandType::Ping => {
                let slave_id = buffer.parse_u32()?;
                let seq = buffer.parse_u32()?;
                Ok(Self::Ping { slave_id, seq })
            }
            HardwareCommandType::ResetAll => Ok(Self::ResetAll),
            HardwareCommandType::SetSectionPower => {
                let section_id = buffer.parse_u32()?;
                let power = buffer.parse()?;
                Ok(Self::SetSectionPower { section_id, power })
            }
            HardwareCommandType::SetSwitchState => {
                let switch_id = buffer.parse()?;
                let state = buffer.parse()?;
                Ok(Self::SetSwitchState { switch_id, state })
            }
        }
    }

    fn deser_serialize(
        &self,
        mut buffer: crate::deser::DeserPayloadWriter,
    ) -> Result<(), crate::deser::DeserError<Self::Error>> {
        match self {
            &Self::Ping { slave_id, seq } => {
                buffer.write_u32(slave_id)?;
                buffer.write_u32(seq)?;
                Ok(())
            }
            Self::ResetAll => Ok(()),
            &Self::SetSectionPower { section_id, power } => {
                buffer.write_u32(section_id)?;
                buffer.write(&power)?;
                Ok(())
            }
            &Self::SetSwitchState { switch_id, state } => {
                buffer.write(&switch_id)?;
                buffer.write(&state)?;
                Ok(())
            }
        }
    }
}
