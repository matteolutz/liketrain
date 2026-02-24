mod section;
use std::collections::HashMap;

pub use section::*;

mod connection;
pub use connection::*;

mod switch;
pub use switch::*;

mod error;
pub use error::*;

use crate::Direction;

#[derive(Debug)]
pub struct Track {
    sections: HashMap<SectionId, Section>,
    switches: HashMap<SwitchId, Switch>,
}

impl Track {
    pub fn new() -> Self {
        Self {
            sections: HashMap::new(),
            switches: HashMap::new(),
        }
    }

    pub fn section(&self, section_id: &SectionId) -> Option<&Section> {
        self.sections.get(section_id)
    }

    pub fn section_mut(&mut self, section_id: &SectionId) -> Option<&mut Section> {
        self.sections.get_mut(section_id)
    }

    pub fn insert_section(&mut self, section: Section) -> SectionId {
        let id = self
            .sections
            .keys()
            .max()
            .map(|id| id.next())
            .unwrap_or_default();

        self.sections.insert(id, section);
        id
    }

    pub fn switch(&self, switch_id: &SwitchId) -> Option<&Switch> {
        self.switches.get(switch_id)
    }

    pub fn switch_mut(&mut self, switch_id: &SwitchId) -> Option<&mut Switch> {
        self.switches.get_mut(switch_id)
    }

    pub fn insert_switch(&mut self, switch: Switch) -> SwitchId {
        let id = self
            .switches
            .keys()
            .max()
            .map(|id| id.next())
            .unwrap_or_default();

        self.switches.insert(id, switch);
        id
    }
}

impl Track {
    pub fn next_sections(
        &self,
        current_section: SectionId,
        direction: Direction,
    ) -> Result<Vec<SectionId>, TrackError> {
        let section = self
            .sections
            .get(&current_section)
            .ok_or(TrackError::SectionNotFound(current_section))?;

        let connection = section.connection(direction);

        let next_sections = match connection {
            Connection::Straight { to } => vec![*to],
            Connection::Switch { switch_id } => {
                let switch = self
                    .switches
                    .get(switch_id)
                    .ok_or(TrackError::SwitchNotFound(*switch_id))?;

                vec![switch.to_left, switch.to_right]
            }
            Connection::SwitchBack { switch_id } => {
                let switch = self
                    .switches
                    .get(switch_id)
                    .ok_or(TrackError::SwitchNotFound(*switch_id))?;

                vec![switch.from]
            }
            Connection::None => vec![],
        };

        Ok(next_sections)
    }
}
