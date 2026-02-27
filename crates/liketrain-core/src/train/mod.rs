mod mode;
pub use mode::*;

use crate::{Direction, Route, SectionEnd, SectionId, SectionTransition};

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
    pub fn from_route(name: String, route: Route) -> Self {
        Self {
            name,
            mode: route.into(),
        }
    }
}

impl Train {
    pub fn get_current_section(&self) -> Option<SectionId> {
        self.mode.get_current_section()
    }

    pub fn get_next_section(&self) -> Option<SectionId> {
        self.mode.get_next_section()
    }

    /// Get the transition from the current section to the next section.
    pub fn get_transition_to_next_section(&self) -> Option<&SectionTransition> {
        match &self.mode {
            TrainDrivingMode::Route {
                route,
                current_via_idx,
                ..
            } => route.transition(*current_via_idx),
        }
    }

    pub fn entered_section(&mut self, section_id: SectionId) {
        let Some(transition) = self.get_transition_to_next_section() else {
            return;
        };

        let expected_next_section = transition.destination();
        if expected_next_section != section_id {
            // TODO: handle this?? maybe switch to manual mode
            return;
        }

        let section_end = transition.destination_section_end();

        match &mut self.mode {
            TrainDrivingMode::Route {
                current_via_idx,
                current_section_direction,
                ..
            } => {
                *current_via_idx += 1;

                match section_end {
                    SectionEnd::End => *current_section_direction = Direction::Backward,
                    SectionEnd::Start => *current_section_direction = Direction::Forward,
                }
            }
        }
    }
}
