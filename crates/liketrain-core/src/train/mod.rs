mod mode;
pub use mode::*;

use crate::SectionId;

#[derive(Debug)]
pub struct Train {
    name: String,

    mode: TrainDrivingMode,
}

impl Train {
    pub fn get_next_section(&self) -> Option<SectionId> {
        self.mode.get_next_section()
    }

    pub fn entered_section(&mut self, section_id: SectionId) {
        let expected_next_section = self.get_next_section();
        if expected_next_section.is_none_or(|expected| expected != section_id) {
            // TODO: handle this?? maybe switch to manual mode
            return;
        }

        match &mut self.mode {
            TrainDrivingMode::Route {
                current_via_idx, ..
            } => {
                *current_via_idx += 1;
            }
        }
    }
}
