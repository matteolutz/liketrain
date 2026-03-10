use crate::{Direction, Route, SectionId};

#[derive(Debug)]
pub enum TrainDrivingMode {
    Route {
        route: Route,

        current_section_direction: Direction,
        current_via_idx: Option<usize>,
    },
}

impl TrainDrivingMode {
    pub fn get_initial_section(&self) -> Option<SectionId> {
        match self {
            Self::Route { route, .. } => route.via(0),
        }
    }

    pub fn get_current_section(&self) -> Option<SectionId> {
        match self {
            Self::Route {
                route,
                current_via_idx,
                ..
            } => current_via_idx.and_then(|idx| route.via(idx)),
        }
    }

    pub fn get_next_section(&self) -> Option<SectionId> {
        match self {
            Self::Route {
                route,
                current_via_idx,
                ..
            } => route.via(current_via_idx.map(|idx| idx + 1).unwrap_or(0)),
        }
    }
}

impl From<Route> for TrainDrivingMode {
    fn from(route: Route) -> Self {
        Self::Route {
            current_section_direction: route.starting_direction(),
            route,
            current_via_idx: None,
        }
    }
}
