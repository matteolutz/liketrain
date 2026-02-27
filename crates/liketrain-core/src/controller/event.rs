use liketrain_hardware::event::HardwareEvent;

use crate::{SectionId, TrainId};

#[derive(Debug)]
pub enum ScheduledEvent {
    TrainEnteredSection {
        train_id: TrainId,
        section_id: SectionId,
    },
    TrainLeftSection {
        train_id: TrainId,
        section_id: SectionId,
    },
}

#[derive(Debug)]
pub enum ControllerEvent {
    Scheduled(ScheduledEvent),
    Hardware(HardwareEvent),
}

impl From<HardwareEvent> for ControllerEvent {
    fn from(value: HardwareEvent) -> Self {
        ControllerEvent::Hardware(value)
    }
}

impl From<ScheduledEvent> for ControllerEvent {
    fn from(value: ScheduledEvent) -> Self {
        ControllerEvent::Scheduled(value)
    }
}
