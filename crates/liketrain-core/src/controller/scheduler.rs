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
        Some(self.cmp(other))
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
    pub fn schedule(&mut self, when: time::Instant, event: impl Into<ScheduledEvent>) {
        let event = event.into();
        self.0.push(TimedEvent { when, event });
    }

    pub fn schedule_now(&mut self, event: impl Into<ScheduledEvent>) {
        self.schedule(time::Instant::now(), event);
    }

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
