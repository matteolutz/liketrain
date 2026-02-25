mod section;
pub use section::*;

pub mod avr;

#[derive(Debug, Copy, Clone)]
pub enum HardwareEvent {
    Pong(u32),
    SectionEvent(SectionEvent),
}
