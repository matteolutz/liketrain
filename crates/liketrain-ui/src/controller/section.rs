use std::collections::VecDeque;

use liketrain_core::{TrainId, hardware::event::HardwareSectionPower};

#[derive(Default, Copy, Clone)]
pub enum UiSectionOccupant {
    #[default]
    None,

    Train(TrainId),
    LeftTrain(TrainId),
}

impl UiSectionOccupant {
    pub fn was_freed(&mut self) {
        if let UiSectionOccupant::Train(train_id) = self {
            *self = UiSectionOccupant::LeftTrain(*train_id);
        }
    }

    pub fn train_id(&self) -> Option<(TrainId, bool)> {
        match self {
            UiSectionOccupant::Train(train_id) => Some((*train_id, false)),
            UiSectionOccupant::LeftTrain(train_id) => Some((*train_id, true)),
            _ => None,
        }
    }
}

impl From<Option<TrainId>> for UiSectionOccupant {
    fn from(value: Option<TrainId>) -> Self {
        match value {
            Some(train_id) => UiSectionOccupant::Train(train_id),
            None => UiSectionOccupant::None,
        }
    }
}

#[derive(Default)]
pub struct UiSectionState {
    pub power: HardwareSectionPower,

    pub occupant: UiSectionOccupant,

    pub reserved_by: Option<TrainId>,
    pub queue: VecDeque<TrainId>,
}
