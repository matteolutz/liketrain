use crate::{command::HardwareCommand, deser::Deser, deser_variant};

deser_variant! {
    HardwareCommandType {
        Ping = 0x1,
        GetSlaves = 0x2,
        SetSectionPower = 0x10,
        SetSwitchState = 0x20,
        ResetAll = 0x30,
    }
}

impl Deser for HardwareCommand {
    type Variant = HardwareCommandType;

    type Error = ();

    fn variant(&self) -> Self::Variant {
        match self {
            Self::Ping { .. } => HardwareCommandType::Ping,
            Self::ResetAll => HardwareCommandType::ResetAll,
            Self::GetSlaves => HardwareCommandType::GetSlaves,
            Self::SetSectionPower { .. } => HardwareCommandType::SetSectionPower,
            Self::SetSwitchState { .. } => HardwareCommandType::SetSwitchState,
        }
    }

    fn deser_deserialize(
        variant: Self::Variant,
        mut buffer: crate::deser::DeserPayloadReader,
    ) -> Result<Self, crate::deser::DeserError<Self::Error>> {
        match variant {
            HardwareCommandType::Ping => {
                let slave_id = buffer.read_u32()?;
                let seq = buffer.read_u32()?;
                Ok(Self::Ping { slave_id, seq })
            }
            HardwareCommandType::GetSlaves => Ok(Self::GetSlaves),
            HardwareCommandType::ResetAll => Ok(Self::ResetAll),
            HardwareCommandType::SetSectionPower => {
                let section_id = buffer.read_u32()?;
                let power = buffer.read()?;
                Ok(Self::SetSectionPower { section_id, power })
            }
            HardwareCommandType::SetSwitchState => {
                let switch_id = buffer.read()?;
                let state = buffer.read()?;
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
            Self::GetSlaves => Ok(()),
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
