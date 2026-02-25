use crate::SectionId;

#[derive(Debug)]
pub struct Route {
    vias: Vec<SectionId>,
}

impl Route {
    pub fn is_closed(&self) -> bool {
        self.vias.len() > 1 && self.vias.first().unwrap() == self.vias.last().unwrap()
    }

    pub fn via(&self, mut idx: usize) -> Option<SectionId> {
        if self.is_closed() {
            idx = idx % (self.vias.len() - 1); // subtract the last element, as it's the same as the first
        }

        self.vias.get(idx).copied()
    }
}
