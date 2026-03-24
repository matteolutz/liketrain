use liketrain_hardware::event::HardwareSectionPower;

use crate::{SectionId, SwitchId, SwitchState};

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
}
