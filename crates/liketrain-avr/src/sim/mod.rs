use liketrain_hardware::event::{HardwareEvent, SectionEvent};

use crate::command::CommandExecutionContext;

pub struct SimTrainRouteVia {
    /// The section id of the route via
    pub section_id: u32,

    /// The time it takes to travel the route via
    pub time: u32,
}

impl From<(u32, u32)> for SimTrainRouteVia {
    fn from((section_id, time): (u32, u32)) -> Self {
        Self { section_id, time }
    }
}

struct SimTrainCurrent {
    route_idx: usize,
    time_progress: u32,
}

impl SimTrainCurrent {
    pub fn new(route_idx: usize) -> Self {
        Self {
            route_idx,
            time_progress: 0,
        }
    }
}

pub struct SimTrain<const N: usize> {
    route: [SimTrainRouteVia; N],

    current: Option<SimTrainCurrent>,

    last_update: Option<u32>,
}

impl<const N: usize> SimTrain<N> {
    pub fn new<V>(route: [V; N]) -> Self
    where
        V: Into<SimTrainRouteVia>,
    {
        let current = (!route.is_empty()).then(|| SimTrainCurrent::new(0));

        Self {
            route: route.map(|via| via.into()),
            current,
            last_update: None,
        }
    }

    fn is_route_closed(&self) -> bool {
        !self.route.is_empty()
            && self.route.first().unwrap().section_id == self.route.last().unwrap().section_id
    }

    fn next_section(&mut self) -> Option<usize> {
        let is_route_closed = self.is_route_closed();

        let current = self.current.as_mut()?;

        if is_route_closed {
            // skip the last
            if current.route_idx + 2 >= self.route.len() {
                *current = SimTrainCurrent::new(0);
                return Some(0);
            }
        }

        if current.route_idx + 1 >= self.route.len() {
            self.current = None;
            None
        } else {
            *current = SimTrainCurrent::new(current.route_idx + 1);
            Some(current.route_idx)
        }
    }

    pub fn update(&mut self, ctx: &mut CommandExecutionContext, millis: u32) {
        let Some(current) = self.current.as_mut() else {
            return;
        };

        let current_section = &self.route[current.route_idx];
        let current_section_id = current_section.section_id;

        let Some(section) = ctx
            .sections
            .iter()
            .find(|section| section.section_id() == current_section_id)
        else {
            return;
        };

        if section.current_power().is_off() {
            return;
        }

        let last_update = self.last_update.get_or_insert(millis);
        let diff = millis - *last_update;
        *last_update = millis;

        current.time_progress += diff;

        if current.time_progress < current_section.time {
            return;
        }

        let new_route_idx = self.next_section();

        if let Some(new_route_idx) = new_route_idx {
            let new_section = self.route[new_route_idx].section_id;

            // free the current section
            ctx.event_list
                .push(HardwareEvent::SectionEvent(SectionEvent::freed(
                    current_section_id,
                )));

            // occupy the new section
            ctx.event_list
                .push(HardwareEvent::SectionEvent(SectionEvent::occupied(
                    new_section,
                )));
        }
    }
}
