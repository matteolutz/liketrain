use crate::{SectionId, SwitchId};

#[derive(Default, Debug)]
pub enum Connection {
    Straight {
        to: SectionId,
    },
    Switch {
        switch_id: SwitchId,
    },
    SwitchBack {
        switch_id: SwitchId,
    },

    #[default]
    None,
}
