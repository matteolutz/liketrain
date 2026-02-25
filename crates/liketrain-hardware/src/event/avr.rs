use crate::event::{HardwareEvent, HardwareSwitchId, HardwareSwitchState, SectionEvent};

#[repr(u8)]
pub enum HardwareEventType {
    Pong = 0,
    SectionEvent = 1,
    SwitchStateChange = 2,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct HardwareEventSwitchStateChange {
    pub switch_id: HardwareSwitchId,
    pub state: HardwareSwitchState,
}

#[repr(C, packed)]
pub union HardwareEventUnion {
    pub pong: u32,
    pub section_event: SectionEvent,
    pub switch_state_change: HardwareEventSwitchStateChange,
}

#[repr(C, packed)]
pub struct HardwareEventStruct {
    pub tag: HardwareEventType,
    pub data: HardwareEventUnion,
}

impl HardwareEventStruct {
    pub fn pong(data: u32) -> Self {
        Self {
            tag: HardwareEventType::Pong,
            data: HardwareEventUnion { pong: data },
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
}

impl From<HardwareEvent> for HardwareEventStruct {
    fn from(value: HardwareEvent) -> Self {
        match value {
            HardwareEvent::Pong(data) => Self::pong(data),
            HardwareEvent::SectionEvent(section_event) => Self::section_event(section_event),
            HardwareEvent::SwitchStateChanged { switch_id, state } => {
                Self::switch_state_change(HardwareEventSwitchStateChange { switch_id, state })
            }
        }
    }
}

impl From<HardwareEventStruct> for HardwareEvent {
    fn from(value: HardwareEventStruct) -> Self {
        match value.tag {
            HardwareEventType::Pong => unsafe { HardwareEvent::Pong(value.data.pong) },
            HardwareEventType::SectionEvent => unsafe {
                HardwareEvent::SectionEvent(value.data.section_event)
            },
            HardwareEventType::SwitchStateChange => unsafe {
                HardwareEvent::SwitchStateChanged {
                    switch_id: value.data.switch_state_change.switch_id,
                    state: value.data.switch_state_change.state,
                }
            },
        }
    }
}
