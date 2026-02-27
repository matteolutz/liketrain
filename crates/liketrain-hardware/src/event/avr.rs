#[cfg(feature = "avr")]
use alloc::string::ToString;

use crate::event::{HardwareEvent, HardwareSwitchId, HardwareSwitchState, SectionEvent};

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum HardwareEventType {
    Pong = 0,
    SectionEvent = 1,
    SwitchStateChange = 2,
    DebugMessage = 99,
}

impl TryFrom<u8> for HardwareEventType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(HardwareEventType::Pong),
            1 => Ok(HardwareEventType::SectionEvent),
            2 => Ok(HardwareEventType::SwitchStateChange),
            99 => Ok(HardwareEventType::DebugMessage),
            _ => Err(()),
        }
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct HardwareEventSwitchStateChange {
    pub switch_id: HardwareSwitchId,
    pub state: HardwareSwitchState,
}

#[repr(C, packed)]
pub union HardwareEventUnion {
    pub pong: (u32, u32),
    pub section_event: SectionEvent,
    pub switch_state_change: HardwareEventSwitchStateChange,
    pub debug_message: u32,
}

#[repr(C, packed)]
pub struct HardwareEventStruct {
    pub tag: HardwareEventType,
    pub data: HardwareEventUnion,
}

impl HardwareEventStruct {
    pub fn pong(slave_id: u32, seq: u32) -> Self {
        Self {
            tag: HardwareEventType::Pong,
            data: HardwareEventUnion {
                pong: (slave_id, seq),
            },
        }
    }

    pub fn section_event(data: SectionEvent) -> Self {
        Self {
            tag: HardwareEventType::SectionEvent,
            data: HardwareEventUnion {
                section_event: data,
            },
        }
    }

    pub fn switch_state_change(data: HardwareEventSwitchStateChange) -> Self {
        Self {
            tag: HardwareEventType::SwitchStateChange,
            data: HardwareEventUnion {
                switch_state_change: data,
            },
        }
    }

    pub fn debug_message(len: u32) -> Self {
        Self {
            tag: HardwareEventType::DebugMessage,
            data: HardwareEventUnion { debug_message: len },
        }
    }
}

impl From<HardwareEvent> for HardwareEventStruct {
    fn from(value: HardwareEvent) -> Self {
        match value {
            HardwareEvent::Pong { slave_id, seq } => Self::pong(slave_id, seq),
            HardwareEvent::SectionEvent(section_event) => Self::section_event(section_event),
            HardwareEvent::SwitchStateChanged { switch_id, state } => {
                Self::switch_state_change(HardwareEventSwitchStateChange { switch_id, state })
            }
            HardwareEvent::DebugMessage { .. } => Self::debug_message(0),
        }
    }
}

impl From<HardwareEventStruct> for HardwareEvent {
    fn from(value: HardwareEventStruct) -> Self {
        match value.tag {
            HardwareEventType::Pong => unsafe {
                HardwareEvent::Pong {
                    slave_id: value.data.pong.0,
                    seq: value.data.pong.1,
                }
            },
            HardwareEventType::SectionEvent => unsafe {
                HardwareEvent::SectionEvent(value.data.section_event)
            },
            HardwareEventType::SwitchStateChange => unsafe {
                HardwareEvent::SwitchStateChanged {
                    switch_id: value.data.switch_state_change.switch_id,
                    state: value.data.switch_state_change.state,
                }
            },
            HardwareEventType::DebugMessage => unsafe {
                HardwareEvent::DebugMessage {
                    message: "".to_string(),
                }
            },
        }
    }
}
