use itertools::Itertools;

use crate::{Direction, SectionEnd, SectionId, SectionTransition, Track};

#[derive(Debug)]
pub struct Route {
    vias: Vec<SectionId>,
    starting_direction: Direction,

    transitions: Vec<SectionTransition>,
}

impl Route {
    pub fn new<I, S>(vias: I, starting_direction: Direction, track: &Track) -> Option<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<SectionId>,
    {
        let vias: Vec<SectionId> = vias.into_iter().map_into().collect();
        let transitions = Self::build_transitions(&vias, starting_direction, track)?;

        Some(Self {
            vias,
            starting_direction,
            transitions,
        })
    }
}

impl Route {
    fn build_transitions(
        vias: &[SectionId],
        starting_direction: Direction,
        track: &Track,
    ) -> Option<Vec<SectionTransition>> {
        let mut section_direction = starting_direction;
        let mut route_transitions = Vec::with_capacity(vias.len());

        for (from_section, to_section) in vias.iter().tuple_windows() {
            let transitions = track.transitions_to(*from_section, section_direction, *to_section);

            let Ok(mut transitions) = transitions else {
                return None;
            };

            let Some(transition) = transitions.pop() else {
                // we can't go from the current section to the next section
                return None;
            };

            let section_end = transition.destination_section_end();

            // depending on which end of the section we're going to, we need to update our direction relative to the section
            match section_end {
                SectionEnd::End => section_direction = Direction::Backward,
                SectionEnd::Start => section_direction = Direction::Forward,
            }

            route_transitions.push(transition);
        }

        Some(route_transitions)
    }

    pub fn starting_direction(&self) -> Direction {
        self.starting_direction
    }

    pub fn is_closed(&self) -> bool {
        self.vias.len() > 1 && self.vias.first().unwrap() == self.vias.last().unwrap()
    }

    pub fn transition(&self, idx: usize) -> Option<&SectionTransition> {
        self.transitions.get(idx)
    }

    pub fn via(&self, mut idx: usize) -> Option<SectionId> {
        if self.is_closed() {
            idx %= self.vias.len() - 1; // subtract the last element, as it's the same as the first
        }

        self.vias.get(idx).copied()
    }

    pub fn pretty_print(&self, track: &Track) -> String {
        let Some(section) = self.vias.first().and_then(|id| track.section(id)) else {
            return "".to_string();
        };

        let mut result = String::new();

        result.push_str(
            format!(
                "section {} (driving {}) ",
                section.name(),
                self.starting_direction
            )
            .as_str(),
        );

        for transition in self.transitions.iter() {
            result.push_str(&transition.pretty_print(track));
            result.push(' ');
        }

        result
    }
}
