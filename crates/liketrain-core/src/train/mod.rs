mod mode;
pub use mode::*;

#[derive(Debug)]
pub struct Train {
    name: String,

    mode: TrainDrivingMode,
}
