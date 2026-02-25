use crate::event::{HardwareEvent, SectionEvent};

#[repr(u8)]
pub enum HardwareEventType {
    Pong = 0,
    SectionEvent = 1,
}

#[repr(C, packed)]
pub union HardwareEventUnion {
    pub pong: u32,
    pub section_event: SectionEvent,
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
}

impl From<HardwareEvent> for HardwareEventStruct {
    fn from(value: HardwareEvent) -> Self {
        match value {
            HardwareEvent::Pong(data) => Self::pong(data),
            HardwareEvent::SectionEvent(section_event) => Self::section_event(section_event),
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
        }
    }
}
