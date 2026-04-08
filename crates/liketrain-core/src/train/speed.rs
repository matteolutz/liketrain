use liketrain_hardware::event::HardwareSectionPower;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum TrainSpeed {
    Slow,
    Medium,
    AlmostFast,

    #[default]
    Fast,
}

impl From<TrainSpeed> for HardwareSectionPower {
    fn from(value: TrainSpeed) -> Self {
        match value {
            TrainSpeed::Slow => HardwareSectionPower::Off,
            TrainSpeed::Medium => HardwareSectionPower::Half,
            TrainSpeed::AlmostFast => HardwareSectionPower::ThreeQuarters,
            TrainSpeed::Fast => HardwareSectionPower::Full,
        }
    }
}
