use liketrain_hardware::{
    command::HardwareCommand,
    deser::{Deser, DeserHelper, DeserSize},
    deser_variant,
};

use crate::mode::SlaveId;

#[derive(Debug)]
pub enum SlaveCommand {
    EventPoll { slave_id: u32 },
    Command(HardwareCommand),
}

impl From<HardwareCommand> for SlaveCommand {
    fn from(value: HardwareCommand) -> Self {
        SlaveCommand::Command(value)
    }
}

deser_variant! {
    SlaveCommandType {
        EventPoll = 0x0,
        Command = 0x10,
    }
}

impl Deser for SlaveCommand {
    type Variant = SlaveCommandType;

    type Error = ();

    fn payload_size(variant: Self::Variant) -> liketrain_hardware::deser::DeserSize {
        match variant {
            SlaveCommandType::EventPoll => size_of::<SlaveId>().into(),
            SlaveCommandType::Command => DeserSize::Irrelevant,
        }
    }

    fn variant(&self) -> Self::Variant {
        match self {
            SlaveCommand::EventPoll { .. } => SlaveCommandType::EventPoll,
            SlaveCommand::Command(_) => SlaveCommandType::Command,
        }
    }

    fn deser_deserialize(
        variant: Self::Variant,
        _payload_size: u32,
        mut buffer: liketrain_hardware::deser::DeserPayloadReader,
    ) -> Result<Self, liketrain_hardware::deser::DeserError<Self::Error>> {
        match variant {
            SlaveCommandType::EventPoll => {
                let slave_id = buffer.parse_u32()?;
                let slave_id: SlaveId = slave_id.try_into()?;

                Ok(SlaveCommand::EventPoll {
                    slave_id: slave_id.as_u32(),
                })
            }
            SlaveCommandType::Command => {
                let command = HardwareCommand::deserialize(buffer.buffer())?;
                Ok(SlaveCommand::Command(command))
            }
        }
    }

    fn deser_serialize(
        &self,
        mut buffer: liketrain_hardware::deser::DeserPayloadWriter,
    ) -> Result<(), liketrain_hardware::deser::DeserError<Self::Error>> {
        match self {
            Self::Command(command) => {
                command.serialize_into(buffer.buffer())?;
                Ok(())
            }
            &Self::EventPoll { slave_id } => {
                buffer.write_u32(slave_id)?;
                Ok(())
            }
        }
    }
}
