use serde::{Deserialize, Serialize, de::Visitor};

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

    pub fn as_u32(&self) -> u32 {
        self.0 as u32
    }

    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl std::fmt::Display for SectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<usize> for SectionId {
    fn from(id: usize) -> Self {
        SectionId(id)
    }
}

impl From<u32> for SectionId {
    fn from(id: u32) -> Self {
        SectionId(id as usize)
    }
}

impl Serialize for SectionId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let formatted = format!("S{}", self.0);
        serializer.serialize_str(&formatted)
    }
}

impl<'de> Deserialize<'de> for SectionId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SectionIdVisitor;

        impl<'de> Visitor<'de> for SectionIdVisitor {
            type Value = SectionId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string in the format \"S<usize>\"")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if let Some(id_str) = v.strip_prefix("S") {
                    let id = id_str
                        .parse::<usize>()
                        .map_err(|_| E::custom("invalid number in section id"))?;
                    Ok(SectionId(id.into()))
                } else {
                    Err(E::custom("section id must start with 'S'"))
                }
            }
        }

        deserializer.deserialize_str(SectionIdVisitor)
    }
}

#[derive(Debug, Clone)]
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
