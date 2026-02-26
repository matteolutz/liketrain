use crate::{Direction, Route, SectionId};

#[derive(Debug)]
pub enum TrainDrivingMode {
    Route {
        route: Route,

        current_section_direction: Direction,
        current_via_idx: usize,
    },
}

impl TrainDrivingMode {
    pub fn get_current_section(&self) -> Option<SectionId> {
        match self {
            Self::Route {
                route,
                current_via_idx,
                ..
            } => route.via(*current_via_idx),
        }
    }

    pub fn get_next_section(&self) -> Option<SectionId> {
        match self {
            Self::Route {
                route,
                current_via_idx,
                ..
            } => route.via(current_via_idx + 1),
        }
    }
}

impl From<Route> for TrainDrivingMode {
    fn from(route: Route) -> Self {
        Self::Route {
            current_section_direction: route.starting_direction(),
            route,
            current_via_idx: 0,
        }
    }
}
