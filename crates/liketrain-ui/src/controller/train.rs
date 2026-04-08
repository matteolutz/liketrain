use liketrain_core::{Route, SectionId, Train, TrainData, TrainSpeed, TrainState};

#[derive(Debug, Clone)]
pub struct UiTrain {
    pub data: TrainData,
    pub route: Option<Route>,

    pub current_section: Option<SectionId>,

    pub speed: TrainSpeed,
    pub state: TrainState,
}

impl From<&Train> for UiTrain {
    fn from(train: &Train) -> Self {
        Self {
            data: train.data().clone(),
            route: train.route().cloned(),
            speed: train.speed(),
            state: train.state(),
            current_section: None,
        }
    }
}
