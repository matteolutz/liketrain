use liketrain_hardware::event::HardwareSectionPower;

use crate::TrainId;

#[derive(Default)]
pub struct SectionState {
    pub(super) occupied: Option<TrainId>,
    pub(super) power: HardwareSectionPower,
}
