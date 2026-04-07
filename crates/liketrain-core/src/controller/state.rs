use liketrain_hardware::event::HardwareSectionPower;

use crate::TrainId;

#[derive(Default)]
pub struct SectionState {
    pub(super) occupied: Option<TrainId>,
    pub(super) power: HardwareSectionPower,
}

impl SectionState {
    pub fn is_occupied(&self) -> bool {
        self.occupied.is_some()
    }

    pub fn occupant(&self) -> Option<TrainId> {
        self.occupied
    }

    pub fn power(&self) -> HardwareSectionPower {
        self.power
    }
}
