use crate::SwitchState;

#[derive(Debug, Clone)]
pub enum ConnectionExpr<'src> {
    Direct {
        to: &'src str,
    },
    Switch {
        switch_name: &'src str,
    },
    SwitchBack {
        switch_name: &'src str,
        required_state: SwitchState,
    },
    None,
}

#[derive(Debug)]
pub struct SectionDef<'src> {
    pub section_name: &'src str,
    pub forward: ConnectionExpr<'src>,
    pub backward: ConnectionExpr<'src>,
}
