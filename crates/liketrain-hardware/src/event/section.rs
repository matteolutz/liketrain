#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HardwareSectionPower {
    Off = 0,
    Quarter = 1,
    Half = 2,
    ThreeQuarters = 3,
    Full = 4,
}

impl HardwareSectionPower {
    pub fn is_off(&self) -> bool {
        *self == HardwareSectionPower::Off
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum SectionEventType {
    Occupied = 0,
    Freed = 1,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct SectionEvent {
    pub section_id: u32,
    pub event_type: SectionEventType,
}

impl SectionEvent {
    pub fn occupied(section_id: u32) -> Self {
        Self {
            section_id,
            event_type: SectionEventType::Occupied,
        }
    }

    pub fn freed(section_id: u32) -> Self {
        Self {
            section_id,
            event_type: SectionEventType::Freed,
        }
    }
}
