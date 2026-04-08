#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub enum TrainState {
    #[default]
    Default,

    Waiting,
}
