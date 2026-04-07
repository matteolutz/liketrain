use crate::event::{HardwareSectionPower, HardwareSwitchId, HardwareSwitchState};

pub mod deser;

#[derive(Debug, Clone)]
pub enum HardwareCommand {
    Ping {
        slave_id: u32,
        seq: u32,
    },

    GetSlaves,

    SetSectionPower {
        section_id: u32,
        power: HardwareSectionPower,
    },

    SetSwitchState {
        switch_id: HardwareSwitchId,
        state: HardwareSwitchState,
    },

    ResetAll,
}
