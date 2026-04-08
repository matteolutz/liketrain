use liketrain_core::{Route, SectionId, Train, TrainData};

#[derive(Debug, Clone)]
pub struct UiTrain {
    pub data: TrainData,
    pub route: Option<Route>,

    pub current_section: Option<SectionId>,
}

impl From<&Train> for UiTrain {
    fn from(value: &Train) -> Self {
        Self {
            data: value.data().clone(),
            route: value.route().cloned(),
            current_section: None,
        }
    }
}
