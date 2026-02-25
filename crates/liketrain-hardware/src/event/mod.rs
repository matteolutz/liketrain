mod section;
pub use section::*;

pub mod avr;

pub type HardwareSwitchId = [u8; 32];

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum HardwareSwitchState {
    Left = 0,
    Right = 1,
}

#[derive(Debug, Copy, Clone)]
pub enum HardwareEvent {
    Pong(u32),
    SectionEvent(SectionEvent),

    SwitchStateChanged {
        switch_id: HardwareSwitchId,
        state: HardwareSwitchState,
    },
}
