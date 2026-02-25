use itertools::Itertools;

use crate::{Direction, SectionEnd, SectionId, Track};

#[derive(Debug)]
pub struct Route {
    vias: Vec<SectionId>,
    starting_direction: Direction,
}

impl Route {
    pub fn new<I, S>(vias: I, starting_direction: Direction) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SectionId>,
    {
        Self {
            vias: vias.into_iter().map_into().collect(),
            starting_direction,
        }
    }
}

impl Route {
    pub fn validate(&self, track: &Track) -> bool {
        let mut section_direction = self.starting_direction;

        for (from_section, to_section) in self.vias.iter().tuple_windows() {
            print!("S{} -> S{} ", from_section, to_section);
            let transitions = track.transitions_to(*from_section, section_direction, *to_section);

            let Ok(transitions) = transitions else {
                return false;
            };

            if transitions.is_empty() {
                // we can't go from the current section to the next section
                return false;
            }

            let transition = &transitions[0];
            println!("{}", transition.pretty_print(track));
            let section_end = transition.destination_section_end();

            // depending on which end of the section we're going to, we need to update our direction relative to the section
            match section_end {
                SectionEnd::End => section_direction = Direction::Backward,
                SectionEnd::Start => section_direction = Direction::Forward,
            }
        }

        true
    }

    pub fn is_closed(&self) -> bool {
        self.vias.len() > 1 && self.vias.first().unwrap() == self.vias.last().unwrap()
    }

    pub fn via(&self, mut idx: usize) -> Option<SectionId> {
        if self.is_closed() {
            idx %= self.vias.len() - 1; // subtract the last element, as it's the same as the first
        }

        self.vias.get(idx).copied()
    }
}
