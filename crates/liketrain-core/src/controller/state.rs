use crate::{SectionId, TrainId};

#[derive(Default)]
pub struct SectionState {
    pub(super) occupied: Option<TrainId>,
}

#[derive(Default)]
pub struct TrainState {
    pub(super) current_section: Option<SectionId>,
}
