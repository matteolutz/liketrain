use std::sync::Arc;

use liketrain_hardware::event::HardwareSwitchState;

use crate::{SectionEnd, SectionId, Track};

#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SwitchId(Arc<String>);

impl SwitchId {
    pub fn matches(&self, other: impl AsRef<str>) -> bool {
        let other = other.as_ref();
        other == self.0.as_ref()
    }
}

impl<T: AsRef<str>> From<T> for SwitchId {
    fn from(value: T) -> Self {
        Self(Arc::new(value.as_ref().to_string()))
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

impl From<HardwareSwitchState> for SwitchState {
    fn from(value: HardwareSwitchState) -> Self {
        match value {
            HardwareSwitchState::Left => Self::Left,
            HardwareSwitchState::Right => Self::Right,
        }
    }
}

impl std::fmt::Display for SwitchState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SwitchState::Left => write!(f, "left"),
            SwitchState::Right => write!(f, "right"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SwitchConnection {
    Section {
        section_id: SectionId,

        /// Which end of the section the switch is connected to.
        section_end: SectionEnd,
    },

    SwitchBack {
        switch_id: SwitchId,
        state: SwitchState,
    },
}

impl SwitchConnection {
    pub fn section(section_id: SectionId, section_end: SectionEnd) -> Self {
        Self::Section {
            section_id,
            section_end,
        }
    }
}

impl SwitchConnection {
    pub const INVALID: Self = Self::Section {
        section_id: SectionId::INVALID,
        section_end: SectionEnd::Start,
    };

    pub fn is_invalid(&self) -> bool {
        self == &Self::INVALID
    }
}

#[derive(Debug)]
pub struct Switch {
    pub(super) name: String,

    pub(super) from: SwitchConnection,

    pub(super) to_left: SwitchConnection,
    pub(super) to_right: SwitchConnection,
}

impl Switch {
    pub fn new(name: String) -> Self {
        Self {
            name,
            from: SwitchConnection::INVALID,
            to_left: SwitchConnection::INVALID,
            to_right: SwitchConnection::INVALID,
        }
    }

    pub fn from(&self) -> &SwitchConnection {
        &self.from
    }

    pub fn set_from(&mut self, from: impl Into<SwitchConnection>) {
        self.from = from.into();
    }

    pub fn set_to(&mut self, to: impl Into<SwitchConnection>, state: SwitchState) {
        match state {
            SwitchState::Left => self.to_left = to.into(),
            SwitchState::Right => self.to_right = to.into(),
        }
    }

    pub fn to(&self, state: SwitchState) -> &SwitchConnection {
        match state {
            SwitchState::Left => &self.to_left,
            SwitchState::Right => &self.to_right,
        }
    }

    /// The section id this switch belongs to.
    /// This is very important to know because we can only power sections.
    pub fn section_id(&self, track: &Track) -> SectionId {
        match &self.from {
            SwitchConnection::Section { section_id, .. } => *section_id,
            SwitchConnection::SwitchBack { switch_id, .. } => {
                let switch = track.switch(switch_id).unwrap();
                switch.section_id(track)
            }
        }
    }

    pub fn pretty_print(&self, track: &Track) -> String {
        let section_id = self.section_id(track);
        let section = track.section(&section_id).unwrap();

        format!("switch {} (powered by section {})", self.name, section.name)
    }
}
