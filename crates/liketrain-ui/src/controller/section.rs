use std::collections::VecDeque;

use liketrain_core::{TrainId, hardware::event::HardwareSectionPower};

#[derive(Default)]
pub struct UiSectionState {
    pub power: HardwareSectionPower,

    pub occupant: Option<TrainId>,

    pub reserved_by: Option<TrainId>,
    pub queue: VecDeque<TrainId>,
}
