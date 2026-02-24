use crate::{Connection, Direction};

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SectionId(usize);

impl SectionId {
    pub const INVALID: Self = Self(usize::MAX);

    pub fn next(&self) -> Self {
        SectionId(self.0 + 1)
    }
}

impl std::fmt::Display for SectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct Section {
    name: String,

    forward: Connection,
    backward: Connection,

    /// The layer of the section in the track layout.
    /// This is only used to render different physical layers of the track seperately.
    layer: usize,
}

impl Section {
    pub fn new(name: String) -> Self {
        Self {
            name,

            forward: Connection::default(),
            backward: Connection::default(),

            layer: 0,
        }
    }

    pub fn set_connection(&mut self, direction: Direction, connection: Connection) {
        match direction {
            Direction::Forward => self.forward = connection,
            Direction::Backward => self.backward = connection,
        }
    }
}

impl Section {
    pub fn connection(&self, direction: Direction) -> &Connection {
        match direction {
            Direction::Forward => &self.forward,
            Direction::Backward => &self.backward,
        }
    }
}
