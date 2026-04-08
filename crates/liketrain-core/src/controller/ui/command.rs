use liketrain_hardware::event::HardwareSectionPower;

use crate::{SectionId, SwitchId, SwitchState, TrainId, TrainSpeed};

#[derive(Debug, Clone)]
pub enum UiCommand {
    SetSectionPower {
        section_id: SectionId,
        power: HardwareSectionPower,
    },

    SetSwitchState {
        switch_id: SwitchId,
        state: SwitchState,
    },

    SetTrainSpeed {
        train_id: TrainId,
        speed: TrainSpeed,
    },
}
