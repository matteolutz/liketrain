use crate::command::HardwareCommand;
use crate::event::HardwareSectionPower;

#[repr(u8)]
pub enum HardwareCommandType {
    Ping = 0,
    SetSectionPower = 1,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct HardwareCommandSetSectionPower {
    pub section_id: u32,
    pub power: HardwareSectionPower,
}

#[repr(C, packed)]
pub union HardwareCommandUnion {
    pub ping: u32,
    pub set_section_power: HardwareCommandSetSectionPower,
}

#[repr(C, packed)]
pub struct HardwareCommandStruct {
    pub tag: HardwareCommandType,
    pub data: HardwareCommandUnion,
}

impl HardwareCommandStruct {
    pub fn ping(id: u32) -> Self {
        Self {
            tag: HardwareCommandType::Ping,
            data: HardwareCommandUnion { ping: id },
        }
    }

    pub fn set_section_power(section_id: u32, power: HardwareSectionPower) -> Self {
        Self {
            tag: HardwareCommandType::SetSectionPower,
            data: HardwareCommandUnion {
                set_section_power: HardwareCommandSetSectionPower { section_id, power },
            },
        }
    }
}

impl From<HardwareCommand> for HardwareCommandStruct {
    fn from(value: HardwareCommand) -> Self {
        match value {
            HardwareCommand::Ping(id) => Self::ping(id),
            HardwareCommand::SetSectionPower { section_id, power } => {
                Self::set_section_power(section_id, power)
            }
        }
    }
}

impl From<HardwareCommandStruct> for HardwareCommand {
    fn from(value: HardwareCommandStruct) -> Self {
        match value.tag {
            HardwareCommandType::Ping => unsafe { HardwareCommand::Ping(value.data.ping) },
            HardwareCommandType::SetSectionPower => unsafe {
                HardwareCommand::SetSectionPower {
                    section_id: value.data.set_section_power.section_id,
                    power: value.data.set_section_power.power,
                }
            },
        }
    }
}
