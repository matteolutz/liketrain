use crate::Route;

#[derive(Debug)]
pub enum TrainDrivingMode {
    Route {
        route: Route,

        current_via_idx: usize,
    },
}
