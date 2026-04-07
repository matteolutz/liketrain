use liketrain_hardware::{
    command::HardwareCommand,
    event::{HardwareSectionPower, SectionEvent},
};

use crate::{SectionId, SwitchId, SwitchState, TrainId};

#[derive(Debug, Clone)]
pub enum UiSectionEvent {
    QueueEnqueued {
        section_id: SectionId,
        train_id: TrainId,
    },
    QueueDequeued {
        section_id: SectionId,
        train_id: TrainId,
    },

    Occupied {
        section_id: SectionId,
        train_id: Option<TrainId>,
    },

    Reserved {
        section_id: SectionId,
        train_id: Option<TrainId>,
    },

    HardwareSectionEvent(SectionEvent),

    SetPower {
        section_id: SectionId,
        power: HardwareSectionPower,
    },
}

#[derive(Debug, Clone)]
pub enum UiSwitchEvent {
    SetState { id: SwitchId, state: SwitchState },
}

#[derive(Debug, Clone)]
pub enum UiTrainEvent {
    Started(HardwareSectionPower),
    Stopped,
}

#[derive(Debug, Clone)]
pub enum UiEvent {
    UiSectionEvent(UiSectionEvent),
    UiSwitchEvent(UiSwitchEvent),
    UiTrainEvent(UiTrainEvent),
    HardwareCommand(HardwareCommand),
}

impl From<UiSectionEvent> for UiEvent {
    fn from(value: UiSectionEvent) -> Self {
        UiEvent::UiSectionEvent(value)
    }
}

impl From<UiSwitchEvent> for UiEvent {
    fn from(value: UiSwitchEvent) -> Self {
        UiEvent::UiSwitchEvent(value)
    }
}

impl From<UiTrainEvent> for UiEvent {
    fn from(value: UiTrainEvent) -> Self {
        UiEvent::UiTrainEvent(value)
    }
}
