use crate::{SectionId, SwitchState};

#[derive(Debug, Clone)]
pub enum ConnectionExpr<'src> {
    /// Direct connection between two sections
    Direct { to: SectionId },

    /// The section goes into a switch
    Switch { switch_name: &'src str },

    /// The section goes into a switch branch
    SwitchBack {
        switch_name: &'src str,
        required_state: SwitchState,
    },

    /// The section has a dead end
    None,
}

#[derive(Debug)]
pub struct SectionDef<'src> {
    pub section_id: SectionId,
    pub forward: ConnectionExpr<'src>,
    pub backward: ConnectionExpr<'src>,
}

#[derive(Debug)]
pub struct SwitchWithState<'src> {
    pub switch_name: &'src str,
    pub state: SwitchState,
}

#[derive(Debug)]
pub struct SwitchConnection<'src> {
    pub from: SwitchWithState<'src>,
    pub to: SwitchWithState<'src>,
}

#[derive(Debug)]
pub enum TrackDefinition<'src> {
    Section(SectionDef<'src>),

    Switch(SwitchConnection<'src>),
}
