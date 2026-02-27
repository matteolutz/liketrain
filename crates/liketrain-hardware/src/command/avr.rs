use crate::command::HardwareCommand;
use crate::event::{HardwareSectionPower, HardwareSwitchId, HardwareSwitchState};

#[repr(u8)]
pub enum HardwareCommandType {
    Ping = 0,
    SetSectionPower = 1,
    SetSwitchState = 2,
    ResetAll = 99,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct HardwareCommandSetSectionPower {
    pub section_id: u32,
    pub power: HardwareSectionPower,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct HardwareCommandSetSwitchState {
    pub switch_id: HardwareSwitchId,
    pub state: HardwareSwitchState,
}

#[repr(C, packed)]
pub union HardwareCommandUnion {
    pub ping: (u32, u32),
    pub set_section_power: HardwareCommandSetSectionPower,
    pub set_switch_state: HardwareCommandSetSwitchState,
    pub reset_all: (),
}

#[repr(C, packed)]
pub struct HardwareCommandStruct {
    pub tag: HardwareCommandType,
    pub data: HardwareCommandUnion,
}

impl HardwareCommandStruct {
    pub fn ping(slave_id: u32, seq: u32) -> Self {
        Self {
            tag: HardwareCommandType::Ping,
            data: HardwareCommandUnion {
                ping: (slave_id, seq),
            },
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

    pub fn set_switch_state(switch_id: HardwareSwitchId, state: HardwareSwitchState) -> Self {
        Self {
            tag: HardwareCommandType::SetSwitchState,
            data: HardwareCommandUnion {
                set_switch_state: HardwareCommandSetSwitchState { switch_id, state },
            },
        }
    }

    pub fn reset_all() -> Self {
        Self {
            tag: HardwareCommandType::ResetAll,
            data: HardwareCommandUnion { reset_all: () },
        }
    }
}

impl From<HardwareCommand> for HardwareCommandStruct {
    fn from(value: HardwareCommand) -> Self {
        match value {
            HardwareCommand::Ping { slave_id, seq } => Self::ping(slave_id, seq),
            HardwareCommand::SetSectionPower { section_id, power } => {
                Self::set_section_power(section_id, power)
            }
            HardwareCommand::SetSwitchState { switch_id, state } => {
                Self::set_switch_state(switch_id, state)
            }
            HardwareCommand::ResetAll => Self::reset_all(),
        }
    }
}

impl From<HardwareCommandStruct> for HardwareCommand {
    fn from(value: HardwareCommandStruct) -> Self {
        match value.tag {
            HardwareCommandType::Ping => unsafe {
                HardwareCommand::Ping {
                    slave_id: value.data.ping.0,
                    seq: value.data.ping.1,
                }
            },
            HardwareCommandType::SetSectionPower => unsafe {
                HardwareCommand::SetSectionPower {
                    section_id: value.data.set_section_power.section_id,
                    power: value.data.set_section_power.power,
                }
            },
            HardwareCommandType::SetSwitchState => unsafe {
                HardwareCommand::SetSwitchState {
                    switch_id: value.data.set_switch_state.switch_id,
                    state: value.data.set_switch_state.state,
                }
            },
            HardwareCommandType::ResetAll => HardwareCommand::ResetAll,
        }
    }
}
