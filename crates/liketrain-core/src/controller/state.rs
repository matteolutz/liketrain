use crate::SectionId;

#[derive(Default)]
pub struct SectionState {
    pub(super) occupied: bool,
}

#[derive(Default)]
pub struct TrainState {
    pub(super) current_section: Option<SectionId>,
}
