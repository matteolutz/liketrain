use std::collections::HashMap;

use thiserror::Error;

use crate::{
    Connection, Direction, Section, SectionEnd, SectionId, Switch, SwitchConnection, SwitchId,
    SwitchState, Track,
    parser::{ConnectionExpr, TrackDefinition},
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

#[derive(Default)]
pub struct Evaluator<'src> {
    section_name_map: HashMap<&'src str, SectionId>,
    switch_name_map: HashMap<&'src str, SwitchId>,
}

impl<'src> Evaluator<'src> {
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

                track.section_mut(&section_id).unwrap().set_connection(
                    direction,
                    Connection::Direct {
                        to: to_section_id,
                        section_end: SectionEnd::end_when(direction),
                    },
                );
            }
            ConnectionExpr::Switch { switch_name } => {
                let (switch_id, switch) = self.get_or_insert_switch(track, switch_name);

                if switch.from() != SwitchConnection::INVALID {
                    return Err(EvaluationError::SwitchFromAlreadyConnected(switch_name));
                }

                switch.set_from(SwitchConnection::section(
                    section_id,
                    SectionEnd::end_when(direction),
                ));
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

                if switch.to(required_state) != SwitchConnection::INVALID {
                    return Err(EvaluationError::SwitchToAlreadyConnected(
                        switch_name,
                        required_state,
                    ));
                }

                switch.set_to(
                    SwitchConnection::section(section_id, SectionEnd::end_when(direction)),
                    required_state,
                );
                track.section_mut(&section_id).unwrap().set_connection(
                    direction,
                    Connection::SwitchBack {
                        switch_id,
                        required_state,
                    },
                );
            }
        }

        Ok(())
    }

    pub fn evaluate(
        mut self,
        track_defs: Vec<TrackDefinition<'src>>,
    ) -> Result<Track, EvaluationError<'src>> {
        let mut track = Track::default();

        for def in track_defs {
            match def {
                TrackDefinition::Section(def) => {
                    let section_id = self.get_or_insert_section_id(&mut track, def.section_name);

                    self.evaluate_connection(
                        &mut track,
                        section_id,
                        def.forward,
                        Direction::Forward,
                    )?;
                    self.evaluate_connection(
                        &mut track,
                        section_id,
                        def.backward,
                        Direction::Backward,
                    )?;
                }
                TrackDefinition::Switch(def) => {
                    let (from_switch_id, _) =
                        self.get_or_insert_switch(&mut track, def.from.switch_name);
                    let (to_switch_id, _) =
                        self.get_or_insert_switch(&mut track, def.to.switch_name);

                    let from_switch = track.switch_mut(&from_switch_id).unwrap();
                    if from_switch.to(def.from.state) != SwitchConnection::INVALID {
                        return Err(EvaluationError::SwitchToAlreadyConnected(
                            def.from.switch_name,
                            def.from.state,
                        ));
                    }
                    from_switch.set_to(
                        SwitchConnection::SwitchBack {
                            switch_id: to_switch_id,
                            state: def.to.state,
                        },
                        def.from.state,
                    );

                    let to_switch = track.switch_mut(&to_switch_id).unwrap();
                    if to_switch.to(def.to.state) != SwitchConnection::INVALID {
                        return Err(EvaluationError::SwitchToAlreadyConnected(
                            def.to.switch_name,
                            def.to.state,
                        ));
                    }
                    to_switch.set_to(
                        SwitchConnection::SwitchBack {
                            switch_id: from_switch_id,
                            state: def.from.state,
                        },
                        def.to.state,
                    );
                }
            }
        }

        Ok(track)
    }
}
