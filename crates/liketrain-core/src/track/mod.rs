mod section;
use std::collections::HashMap;

pub use section::*;

mod connection;
pub use connection::*;

mod switch;
pub use switch::*;

mod transition;
pub use transition::*;

mod error;
pub use error::*;

use crate::Direction;

#[derive(Debug, Default)]
pub struct Track {
    sections: HashMap<SectionId, Section>,
    switches: HashMap<SwitchId, Switch>,
}

impl Track {
    pub fn section(&self, section_id: &SectionId) -> Option<&Section> {
        self.sections.get(section_id)
    }

    pub fn section_mut(&mut self, section_id: &SectionId) -> Option<&mut Section> {
        self.sections.get_mut(section_id)
    }

    pub fn section_id(&self, section_name: &str) -> Option<SectionId> {
        self.sections
            .iter()
            .find_map(|(id, section)| (section.name == section_name).then_some(*id))
    }

    pub fn insert_section(
        &mut self,
        section_id: SectionId,
        section: Section,
    ) -> Result<(), TrackError> {
        if self.sections.contains_key(&section_id) {
            return Err(TrackError::SectionAlreadyExists(section_id));
        }

        self.sections.insert(section_id, section);
        Ok(())
    }

    pub fn switch(&self, switch_id: &SwitchId) -> Option<&Switch> {
        self.switches.get(switch_id)
    }

    pub fn switch_mut(&mut self, switch_id: &SwitchId) -> Option<&mut Switch> {
        self.switches.get_mut(switch_id)
    }

    pub fn switch_section_id(&self, switch_id: &SwitchId) -> Option<SectionId> {
        self.switch(switch_id).map(|switch| switch.section_id(self))
    }

    pub fn insert_switch(
        &mut self,
        switch_id: impl Into<SwitchId>,
        switch: Switch,
    ) -> Result<(), TrackError> {
        let switch_id = switch_id.into();

        if self.switches.contains_key(&switch_id) {
            return Err(TrackError::SwitchAlreadyExists(switch_id));
        }

        self.switches.insert(switch_id, switch);
        Ok(())
    }
}

impl Track {
    fn make_switch_transition(
        &self,
        switch_connection: &SwitchConnection,
    ) -> Vec<SectionTransition> {
        match switch_connection {
            SwitchConnection::Section {
                section_id,
                section_end,
            } => {
                vec![SectionTransition::direct(*section_id, *section_end)]
            }

            SwitchConnection::SwitchBack { switch_id, state } => {
                let Some(switch) = self.switches.get(switch_id) else {
                    return vec![];
                };

                self.make_switch_transition(switch.from())
                    .into_iter()
                    .map(|trans| SectionTransition::switch_back(switch_id.clone(), *state, trans))
                    .collect()
            }
        }
    }

    /// Get the transitions from a section into a neighbouring section in a given direction.
    pub fn transitions_to(
        &self,
        current_section: SectionId,
        direction: Direction,
        target_section: SectionId,
    ) -> Result<Vec<SectionTransition>, TrackError> {
        self.transitions(current_section, direction)
            .map(|transitions| {
                transitions
                    .into_iter()
                    .filter(|trans| trans.destination() == target_section)
                    .collect()
            })
    }

    /// Get all transitions from a section in a given direction.
    pub fn transitions(
        &self,
        current_section: SectionId,
        direction: Direction,
    ) -> Result<Vec<SectionTransition>, TrackError> {
        let section = self
            .sections
            .get(&current_section)
            .ok_or(TrackError::SectionNotFound(current_section))?;

        let connection = section.connection(direction);

        let next_sections = match connection {
            Connection::Direct {
                to,
                section_end: end,
            } => vec![SectionTransition::direct(*to, *end)],

            Connection::Switch { switch_id } => {
                let switch = self
                    .switches
                    .get(switch_id)
                    .ok_or(TrackError::SwitchNotFound(switch_id.clone()))?;

                let left_transitions = self
                    .make_switch_transition(switch.to(SwitchState::Left))
                    .into_iter()
                    .map(|trans| {
                        SectionTransition::switch(switch_id.clone(), SwitchState::Left, trans)
                    });
                let right_transitions = self
                    .make_switch_transition(switch.to(SwitchState::Right))
                    .into_iter()
                    .map(|trans| {
                        SectionTransition::switch(switch_id.clone(), SwitchState::Right, trans)
                    });

                left_transitions.chain(right_transitions).collect()
            }

            Connection::SwitchBack {
                switch_id,
                required_state,
            } => {
                let switch = self
                    .switches
                    .get(switch_id)
                    .ok_or(TrackError::SwitchNotFound(switch_id.clone()))?;

                self.make_switch_transition(switch.from())
                    .into_iter()
                    .map(|trans| {
                        SectionTransition::switch_back(switch_id.clone(), *required_state, trans)
                    })
                    .collect()
            }

            Connection::None => vec![],
        };

        Ok(next_sections)
    }
}
