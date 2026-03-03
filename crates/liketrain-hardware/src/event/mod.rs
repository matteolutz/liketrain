mod section;
pub use section::*;

pub mod deser;

pub const HARDWARE_SWITCH_ID_MAX_LEN: usize = 32;
pub type HardwareSwitchId = [u8; 32];

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum HardwareSwitchState {
    Left = 0,
    Right = 1,
}

#[derive(Debug, Clone)]
pub enum HardwareEvent {
    Pong {
        slave_id: u32,
        seq: u32,
    },

    Slaves {
        n_slaves: u32,
    },

    SectionEvent(SectionEvent),

    SwitchStateChanged {
        switch_id: HardwareSwitchId,
        state: HardwareSwitchState,
    },
}
