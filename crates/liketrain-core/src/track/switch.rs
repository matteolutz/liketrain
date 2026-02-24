use crate::SectionId;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SwitchId(usize);

impl SwitchId {
    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

impl From<usize> for SwitchId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for SwitchId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SwitchState {
    Left,
    Right,
}

impl std::fmt::Display for SwitchState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SwitchState::Left => write!(f, "left"),
            SwitchState::Right => write!(f, "right"),
        }
    }
}

#[derive(Debug)]
pub struct Switch {
    name: String,

    pub(super) from: SectionId,

    pub(super) to_left: SectionId,
    pub(super) to_right: SectionId,
}

impl Switch {
    pub fn new(name: String) -> Self {
        Self {
            name,
            from: SectionId::INVALID,
            to_left: SectionId::INVALID,
            to_right: SectionId::INVALID,
        }
    }

    pub fn from(&self) -> SectionId {
        self.from
    }

    pub fn set_from(&mut self, from: SectionId) {
        self.from = from;
    }

    pub fn set_to(&mut self, to: SectionId, state: SwitchState) {
        match state {
            SwitchState::Left => self.to_left = to,
            SwitchState::Right => self.to_right = to,
        }
    }

    pub fn to(&self, state: SwitchState) -> SectionId {
        match state {
            SwitchState::Left => self.to_left,
            SwitchState::Right => self.to_right,
        }
    }
}
