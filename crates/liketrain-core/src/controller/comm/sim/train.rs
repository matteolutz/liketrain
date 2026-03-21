use std::{collections::HashMap, thread::current, time};

use liketrain_hardware::event::{HardwareEvent, HardwareSectionPower, SectionEvent};

use crate::SectionId;

#[derive(Clone)]
pub struct SimTrainVia {
    section_id: SectionId,
    time_to_travel: time::Duration,
}

#[derive(Clone)]
pub struct SimTrainCurrentVia {
    idx: usize,
    last_update: time::Instant,
}

impl SimTrainCurrentVia {
    pub fn entered_now(idx: usize) -> Self {
        Self {
            idx,
            last_update: time::Instant::now(),
        }
    }

    pub fn next(&self, vias: &[SimTrainVia]) -> Self {
        Self::entered_now((self.idx + 1) % vias.len())
    }
}

#[derive(Clone)]
pub struct SimTrain {
    vias: Vec<SimTrainVia>,

    current_via: Option<SimTrainCurrentVia>,
}

impl SimTrain {
    pub fn update(
        &mut self,
        section_states: &HashMap<SectionId, HardwareSectionPower>,
        events: &mut Vec<HardwareEvent>,
    ) {
        let Some(current_via) = self.current_via.as_mut() else {
            return;
        };

        let delta = current_via.last_update.elapsed();

        current_via.last_update = time::Instant::now();

        let current_section = &self.vias[current_via.idx];
        if current_via.entered_at.elapsed() < current_section.time_to_travel {
            return;
        }

        events.push(HardwareEvent::SectionEvent(SectionEvent::occupied(
            current_section.section_id.as_u32(),
        )));

        *current_via = current_via.next(&self.vias);

        let new_section = &self.vias[current_via.idx];
        events.push(HardwareEvent::SectionEvent(SectionEvent::occupied(
            new_section.section_id.as_u32(),
        )));
    }
}
