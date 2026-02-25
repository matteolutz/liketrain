use crate::{SectionEnd, SectionId, SwitchId, SwitchState, Track};

#[derive(Debug, Clone)]
pub enum SectionTransition {
    Direct {
        section_id: SectionId,

        /// The section end that the train is transitioning to.
        /// This is important, because it may require reversing the direction of the train.
        section_end: SectionEnd,
    },

    Switch {
        switch_id: SwitchId,
        state: SwitchState,
        to: Box<SectionTransition>,
    },

    SwitchBack {
        switch_id: SwitchId,
        state: SwitchState,
        to: Box<SectionTransition>,
    },
}

impl SectionTransition {
    pub fn direct(id: SectionId, section_end: SectionEnd) -> Self {
        Self::Direct {
            section_id: id,
            section_end,
        }
    }

    pub fn switch(switch_id: SwitchId, state: SwitchState, to: SectionTransition) -> Self {
        Self::Switch {
            switch_id,
            state,
            to: Box::new(to),
        }
    }

    pub fn switch_back(switch_id: SwitchId, state: SwitchState, to: SectionTransition) -> Self {
        Self::SwitchBack {
            switch_id,
            state,
            to: Box::new(to),
        }
    }

    pub fn destination(&self) -> SectionId {
        match self {
            Self::Direct { section_id, .. } => *section_id,
            Self::Switch { to, .. } => to.destination(),
            Self::SwitchBack { to, .. } => to.destination(),
        }
    }

    pub fn destination_section_end(&self) -> SectionEnd {
        match self {
            Self::Direct { section_end, .. } => *section_end,
            Self::Switch { to, .. } => to.destination_section_end(),
            Self::SwitchBack { to, .. } => to.destination_section_end(),
        }
    }
}

impl SectionTransition {
    pub fn pretty_print(&self, track: &Track) -> String {
        match self {
            Self::Direct {
                section_id,
                section_end,
            } => {
                let section = track.section(section_id).unwrap();
                format!("-> section {} ({})", section.name, section_end)
            }
            Self::Switch {
                switch_id,
                state,
                to,
            } => {
                let switch = track.switch(switch_id).unwrap();
                format!(
                    "-> {} to {} branch {}",
                    switch.pretty_print(track),
                    state,
                    to.pretty_print(track)
                )
            }
            Self::SwitchBack {
                switch_id,
                state,
                to,
            } => {
                let switch = track.switch(switch_id).unwrap();
                format!(
                    "-> {} from {} branch {}",
                    switch.pretty_print(track),
                    state,
                    to.pretty_print(track)
                )
            }
        }
    }

    pub fn pretty_print_iter<'a>(
        iter: impl IntoIterator<Item = &'a Self>,
        track: &Track,
    ) -> String {
        let mut result = String::new();

        for i in iter.into_iter() {
            result.push_str(&i.pretty_print(track));
            result.push('\n');
        }

        result
    }
}
