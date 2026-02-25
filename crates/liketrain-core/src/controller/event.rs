use liketrain_hardware::event::HardwareEvent;

pub enum ScheduledEvent {}

pub enum ControllerEvent {
    Scheduled(ScheduledEvent),
    Hardware(HardwareEvent),
}

impl From<HardwareEvent> for ControllerEvent {
    fn from(value: HardwareEvent) -> Self {
        ControllerEvent::Hardware(value)
    }
}

impl From<ScheduledEvent> for ControllerEvent {
    fn from(value: ScheduledEvent) -> Self {
        ControllerEvent::Scheduled(value)
    }
}
