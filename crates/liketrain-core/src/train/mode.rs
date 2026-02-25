use crate::{Route, SectionId};

#[derive(Debug)]
pub enum TrainDrivingMode {
    Route {
        route: Route,

        current_via_idx: usize,
    },
}

impl TrainDrivingMode {
    pub fn get_next_section(&self) -> Option<SectionId> {
        match self {
            Self::Route {
                route,
                current_via_idx,
            } => route.via(current_via_idx + 1),
        }
    }
}
