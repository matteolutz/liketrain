use crate::event::{HardwareSectionPower, HardwareSwitchId, HardwareSwitchState};

pub mod avr;

pub mod deser;

#[derive(Debug)]
pub enum HardwareCommand {
    Ping {
        slave_id: u32,
        seq: u32,
    },

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
