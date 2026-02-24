use crate::SectionId;

#[derive(Debug)]
pub struct Route {
    vias: Vec<SectionId>,
}

impl Route {
    pub fn is_closed(&self) -> bool {
        self.vias.len() > 1 && self.vias.first().unwrap() == self.vias.last().unwrap()
    }
}
