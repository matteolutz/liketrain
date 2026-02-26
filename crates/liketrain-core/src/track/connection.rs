use crate::{SectionEnd, SectionId, SwitchId, SwitchState};

#[derive(Default, Debug, PartialEq, Eq)]
pub enum Connection {
    Direct {
        to: SectionId,

        /// The end of the current section that is connected to `to`.
        section_end: SectionEnd,
    },

    Switch {
        switch_id: SwitchId,
    },

    SwitchBack {
        switch_id: SwitchId,
        required_state: SwitchState,
    },

    #[default]
    None,
}
