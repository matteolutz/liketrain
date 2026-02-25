#![no_std]
#![no_main]

use liketrain_hardware::{command::HardwareCommand, event::HardwareEvent};
use panic_halt as _;

use crate::serial::UsartExt;

mod serial;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut serial = arduino_hal::default_serial!(dp, pins, 115200);

    loop {
        if let Ok(command) = serial.try_read_command() {
            match command {
                HardwareCommand::Ping(id) => {
                    let response = HardwareEvent::Pong(id);
                    serial.write_event(response);
                }
                _ => {}
            }
        }
    }
}
