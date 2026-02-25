use std::{
    collections::BinaryHeap,
    time::{self, Duration},
};

use crate::ScheduledEvent;

pub struct TimedEvent {
    when: time::Instant,
    event: ScheduledEvent,
}

impl Ord for TimedEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.when.cmp(&self.when)
    }
}

impl PartialOrd for TimedEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        other.when.partial_cmp(&self.when)
    }
}

impl PartialEq for TimedEvent {
    fn eq(&self, other: &Self) -> bool {
        self.when == other.when
    }
}

impl Eq for TimedEvent {}

#[derive(Default)]
pub struct Scheduler(BinaryHeap<TimedEvent>);

impl Scheduler {
    pub fn next_event_duration(&self) -> Option<Duration> {
        self.0.peek().map(|event| {
            let now = time::Instant::now();
            if event.when <= now {
                Duration::ZERO
            } else {
                event.when - now
            }
        })
    }

    pub fn next_event(&mut self) -> Option<ScheduledEvent> {
        let now = time::Instant::now();

        if self.0.is_empty() {
            return None;
        }

        if self.0.peek().is_some_and(|event| event.when > now) {
            return None;
        }

        let event = self.0.pop().unwrap();
        Some(event.event)
    }
}
