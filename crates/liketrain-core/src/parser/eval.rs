use std::collections::HashMap;

use thiserror::Error;

use crate::{
    Connection, Direction, Section, SectionId, Switch, SwitchId, SwitchState, Track,
    parser::{ConnectionExpr, SectionDef},
};

#[derive(Error, Debug)]
pub enum EvaluationError<'src> {
    #[error("Section '{0}' was already defined")]
    SectionWasAlreadyDefined(&'src str),

    #[error("Switch '{0}' is already connected")]
    SwitchFromAlreadyConnected(&'src str),

    #[error("The {1} back of switch '{0}' is already connected")]
    SwitchToAlreadyConnected(&'src str, SwitchState),
}

pub struct Evaluator<'src> {
    section_name_map: HashMap<&'src str, SectionId>,
    switch_name_map: HashMap<&'src str, SwitchId>,
}

impl<'src> Evaluator<'src> {
    pub fn new() -> Self {
        Self {
            section_name_map: HashMap::new(),
            switch_name_map: HashMap::new(),
        }
    }

    fn get_or_insert_section_id(&mut self, track: &mut Track, name: &'src str) -> SectionId {
        if let Some(section_id) = self.section_name_map.get(name) {
            return *section_id;
        }

        let section = Section::new(name.to_string());
        let section_id = track.insert_section(section);
        self.section_name_map.insert(name, section_id);
        section_id
    }

    fn get_or_insert_switch<'a>(
        &mut self,
        track: &'a mut Track,
        name: &'src str,
    ) -> (SwitchId, &'a mut Switch) {
        if let Some(switch_id) = self.switch_name_map.get(name) {
            return (*switch_id, track.switch_mut(switch_id).unwrap());
        }

        let switch = Switch::new(name.to_string());
        let switch_id = track.insert_switch(switch);
        self.switch_name_map.insert(name, switch_id);
        (switch_id, track.switch_mut(&switch_id).unwrap())
    }

    fn evaluate_connection(
        &mut self,
        track: &mut Track,
        section_id: SectionId,
        connection: ConnectionExpr<'src>,
        direction: Direction,
    ) -> Result<(), EvaluationError<'src>> {
        match connection {
            ConnectionExpr::None => {
                track
                    .section_mut(&section_id)
                    .unwrap()
                    .set_connection(direction, Connection::None);
            }
            ConnectionExpr::Direct { to } => {
                let to_section_id = self.get_or_insert_section_id(track, to);
                track
                    .section_mut(&section_id)
                    .unwrap()
                    .set_connection(direction, Connection::Straight { to: to_section_id });
            }
            ConnectionExpr::Switch { switch_name } => {
                let (switch_id, switch) = self.get_or_insert_switch(track, switch_name);

                if switch.from() != SectionId::INVALID {
                    return Err(EvaluationError::SwitchFromAlreadyConnected(switch_name));
                }

                switch.set_from(section_id);
                track
                    .section_mut(&section_id)
                    .unwrap()
                    .set_connection(direction, Connection::Switch { switch_id });
            }
            ConnectionExpr::SwitchBack {
                switch_name,
                required_state,
            } => {
                let (switch_id, switch) = self.get_or_insert_switch(track, switch_name);

                if switch.to(required_state) != SectionId::INVALID {
                    return Err(EvaluationError::SwitchToAlreadyConnected(
                        switch_name,
                        required_state,
                    ));
                }

                switch.set_to(section_id, required_state);
                track
                    .section_mut(&section_id)
                    .unwrap()
                    .set_connection(direction, Connection::SwitchBack { switch_id });
            }
        }

        Ok(())
    }

    pub fn evaluate(
        mut self,
        section_defs: Vec<SectionDef<'src>>,
    ) -> Result<Track, EvaluationError<'src>> {
        let mut track = Track::new();

        for def in section_defs {
            let section_id = self.get_or_insert_section_id(&mut track, def.section_name);

            self.evaluate_connection(&mut track, section_id, def.forward, Direction::Forward)?;
            self.evaluate_connection(&mut track, section_id, def.backward, Direction::Backward)?;
        }

        Ok(track)
    }
}
