use std::{collections::HashMap, time};

use liketrain_hardware::event::{
    HardwareEvent, HardwareSectionPower, SectionEvent, SectionEventType,
};

use crate::{Route, SectionId, SectionTransition, SwitchId, SwitchState, Track};

#[derive(Clone)]
pub struct SimTrainVia {
    section_id: SectionId,

    /// The transition into this section, if any.
    transition: Option<SectionTransition>,

    time_to_travel: time::Duration,
}

#[derive(Clone)]
struct SimTrainCurrentViaOn {
    idx: usize,
    last_update: time::Instant,
    time_traveled: time::Duration,
}

impl SimTrainCurrentViaOn {
    fn get_power_multiplier(current_power: HardwareSectionPower) -> f64 {
        match current_power {
            HardwareSectionPower::Off => 0.0,
            HardwareSectionPower::Quarter => 0.25,
            HardwareSectionPower::Half => 0.5,
            HardwareSectionPower::ThreeQuarters => 0.75,
            HardwareSectionPower::Full => 1.0,
        }
    }

    pub fn update_now(&mut self, current_power: HardwareSectionPower) {
        let delta = self.last_update.elapsed();
        self.last_update = time::Instant::now();
        self.time_traveled += delta.mul_f64(Self::get_power_multiplier(current_power));
    }
}

#[derive(Clone)]
enum SimTrainCurrentVia {
    On(SimTrainCurrentViaOn),

    Transitioning { to: usize },

    Stopped,
}

impl Default for SimTrainCurrentVia {
    fn default() -> Self {
        Self::Transitioning { to: 0 }
    }
}

#[derive(Clone)]
pub struct SimTrain {
    vias: Vec<SimTrainVia>,

    current_via: SimTrainCurrentVia,
}

impl SimTrain {
    pub fn new<I, V>(vias: I) -> Self
    where
        I: IntoIterator<Item = V>,
        V: Into<SimTrainVia>,
    {
        let vias = vias.into_iter().map(|v| v.into()).collect::<Vec<_>>();

        let current_via = if vias.is_empty() {
            SimTrainCurrentVia::Stopped
        } else {
            SimTrainCurrentVia::Transitioning { to: 0 }
        };

        Self { vias, current_via }
    }

    /// Get a `SimTrain` from a `Route` and `Track`, with the given speed.
    ///
    /// * `speed` - the speed of the train in meters per second (already in respect to the tracks scale)
    pub fn from_route(route: &Route, track: &Track, speed: f32) -> Self {
        let vias = route
            .vias()
            .iter()
            .copied()
            .enumerate()
            .map(|(idx, section_id)| {
                let section_geo = track.section_geo(&section_id).expect("When using .from_route() please make sure each Section is linked to a TrackSectionGeometry");
                let seconds_to_travel = (section_geo.length / speed) as u64;

                let transition = (idx > 0).then(|| route.transition(idx - 1).cloned()).flatten();

                SimTrainVia {
                    section_id,
                    transition,
                    time_to_travel: time::Duration::from_secs(seconds_to_travel),
                }
            })
            .collect::<Vec<_>>();

        let current_via = if vias.is_empty() {
            SimTrainCurrentVia::Stopped
        } else {
            SimTrainCurrentVia::Transitioning { to: 0 }
        };

        Self { vias, current_via }
    }

    pub(super) fn update(
        &mut self,
        section_states: &HashMap<SectionId, HardwareSectionPower>,
        switch_states: &HashMap<SwitchId, SwitchState>,
        events: &mut Vec<HardwareEvent>,
    ) {
        match &mut self.current_via {
            SimTrainCurrentVia::Stopped => {}
            SimTrainCurrentVia::Transitioning { to } => {
                let next_section = &self.vias[*to];
                let next_section_power = section_states
                    .get(&next_section.section_id)
                    .copied()
                    .unwrap_or_default();

                // check that switches are set correctly
                if let Some(transition) = next_section.transition.as_ref() {
                    for switch_change in transition.required_switch_changes() {
                        let actual_switch_state = switch_states
                            .get(&switch_change.switch_id)
                            .copied()
                            .unwrap_or_default();

                        if actual_switch_state != switch_change.required_state {
                            return;
                        }
                    }
                }

                // check that the next section is powered on
                if next_section_power.is_off() {
                    return;
                }

                events.push(HardwareEvent::SectionEvent(SectionEvent {
                    section_id: next_section.section_id.as_u32(),
                    event_type: SectionEventType::Occupied,
                }));

                self.current_via = SimTrainCurrentVia::On(SimTrainCurrentViaOn {
                    idx: *to,
                    last_update: time::Instant::now(),
                    time_traveled: time::Duration::from_secs(0),
                });
            }
            SimTrainCurrentVia::On(current_via) => {
                let current_section = &self.vias[current_via.idx];
                let current_section_power = section_states
                    .get(&current_section.section_id)
                    .copied()
                    .unwrap_or_default();

                current_via.update_now(current_section_power);

                if current_via.time_traveled < current_section.time_to_travel {
                    return;
                }

                if current_section_power.is_off() {
                    return;
                }

                // we are at the end of the current section
                events.push(HardwareEvent::SectionEvent(SectionEvent::freed(
                    current_section.section_id.as_u32(),
                )));

                let next_via_idx = (current_via.idx + 1) % self.vias.len();
                self.current_via = SimTrainCurrentVia::Transitioning { to: next_via_idx };
            }
        }
    }
}
