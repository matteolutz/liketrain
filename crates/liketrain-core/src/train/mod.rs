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

#[derive(Debug, Clone)]
pub struct TrainData {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Train {
    data: TrainData,

    mode: TrainDrivingMode,
}

impl Train {
    pub fn from_route(name: impl Into<String>, route: Route) -> Self {
        Self {
            data: TrainData { name: name.into() },
            mode: route.into(),
        }
    }
}

impl Train {
    pub fn data(&self) -> &TrainData {
        &self.data
    }

    pub fn name(&self) -> &str {
        &self.data.name
    }

    pub fn driving_mode(&self) -> &TrainDrivingMode {
        &self.mode
    }

    pub fn get_initial_section(&self) -> Option<SectionId> {
        self.mode.get_initial_section()
    }

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
            } => current_via_idx.and_then(|idx| route.transition(idx)),
        }
    }

    pub fn entered_section(&mut self, section_id: SectionId) {
        let transition = self.get_transition_to_next_section().cloned();

        let expected_next_section = transition.as_ref().map(|trans| trans.destination());

        if expected_next_section.is_some_and(|id| id != section_id) {
            // TODO: handle this?? maybe switch to manual mode
            return;
        }

        match &mut self.mode {
            TrainDrivingMode::Route {
                current_via_idx,
                current_section_direction,
                route,
            } => {
                match current_via_idx {
                    Some(idx) => *idx += 1,
                    None => *current_via_idx = Some(0),
                }

                match transition.map(|trans| trans.destination_section_end()) {
                    None => *current_section_direction = route.starting_direction(),
                    Some(section_end) => match section_end {
                        SectionEnd::End => *current_section_direction = Direction::Backward,
                        SectionEnd::Start => *current_section_direction = Direction::Forward,
                    },
                };
            }
        }
    }
}
