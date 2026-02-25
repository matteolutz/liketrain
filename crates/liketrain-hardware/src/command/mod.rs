use crate::event::HardwareSectionPower;

pub mod avr;

#[derive(Debug)]
pub enum HardwareCommand {
    Ping(u32),
    SetSectionPower {
        section_id: u32,
        power: HardwareSectionPower,
    },
}
