use crate::{Connection, Direction};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SectionEnd {
    /// The start of the section. Going forward means getting to the end.
    Start,

    /// The end of the section. Going backward means getting to the start.
    End,
}

impl SectionEnd {
    /// The end you will reach, when driving in the given direction.
    pub fn end_when(direction: Direction) -> Self {
        match direction {
            Direction::Forward => SectionEnd::End,
            Direction::Backward => SectionEnd::Start,
        }
    }
}

impl std::fmt::Display for SectionEnd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SectionEnd::Start => write!(f, "start"),
            SectionEnd::End => write!(f, "end"),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SectionId(usize);

impl SectionId {
    pub const INVALID: Self = Self(usize::MAX);

    pub fn new(id: usize) -> Self {
        SectionId(id)
    }

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
    pub(super) name: String,

    forward: Connection,
    backward: Connection,
}

impl Section {
    pub fn new(name: String) -> Self {
        Self {
            name,

            forward: Connection::default(),
            backward: Connection::default(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
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
