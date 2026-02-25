mod mode;
pub use mode::*;

use crate::{SectionId, Track};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrainId(usize);

impl TrainId {
    pub fn new(id: usize) -> Self {
        TrainId(id)
    }
}

impl std::fmt::Display for TrainId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<usize> for TrainId {
    fn from(id: usize) -> Self {
        Self(id)
    }
}

impl From<u32> for TrainId {
    fn from(id: u32) -> Self {
        Self(id as usize)
    }
}

#[derive(Debug)]
pub struct Train {
    name: String,

    mode: TrainDrivingMode,
}

impl Train {
    pub fn validate_route(&self, track: &Track) -> bool {
        match &self.mode {
            TrainDrivingMode::Route { route, .. } => route.validate(track),
        }
    }

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
