use liketrain_hardware::{
    command::{HardwareCommand, avr::HardwareCommandStruct},
    event::{HardwareEvent, avr::HardwareEventStruct},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SlaveId(u32);

impl SlaveId {
    pub const MASTER_ID: u32 = 0;

    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for SlaveId {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value == Self::MASTER_ID {
            Err(())
        } else {
            Ok(SlaveId(value))
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum LiketrainMode {
    Master,
    Slave { slave_id: SlaveId },
}

impl LiketrainMode {
    pub fn is_master(&self) -> bool {
        matches!(self, LiketrainMode::Master)
    }

    pub fn get_slave_id(&self) -> u32 {
        match self {
            LiketrainMode::Master => SlaveId::MASTER_ID,
            LiketrainMode::Slave { slave_id } => slave_id.as_u32(),
        }
    }
}

pub enum SlaveCommand {
    EventPoll { slave_id: SlaveId },
    Command(HardwareCommandStruct),
}

impl From<HardwareCommand> for SlaveCommand {
    fn from(value: HardwareCommand) -> Self {
        Self::Command(value.into())
    }
}

pub enum SlaveResponse {
    EventCount { count: u32 },
    Event(HardwareEventStruct),
}

impl From<HardwareEvent> for SlaveResponse {
    fn from(value: HardwareEvent) -> Self {
        Self::Event(value.into())
    }
}
