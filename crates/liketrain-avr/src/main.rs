#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

extern crate alloc;

use core::sync::atomic::AtomicBool;

use arduino_hal::Usart;
use avr_device::interrupt::Mutex;
use embedded_alloc::LlffHeap;
use liketrain_hardware::{command::HardwareCommand, event::HardwareEvent};
use panic_halt as _;

use crate::{rs485::Rs485, serial::UsartExt};

mod mode;
mod pcint;
mod rs485;
mod serial;
mod track;

#[global_allocator]
static HEAP: LlffHeap = LlffHeap::empty();
const HEAP_SIZE: usize = 1024;

static PCINT_TRAMPOLINES: Mutex<[Option<fn() -> ()>; 10]> = Mutex::new([None; 10]);

static TOGGLED: AtomicBool = AtomicBool::new(false);

#[avr_device::interrupt(atmega2560)]
fn PCINT0() {
    let current = TOGGLED.load(core::sync::atomic::Ordering::SeqCst);
    TOGGLED.store(!current, core::sync::atomic::Ordering::SeqCst);
}

#[arduino_hal::entry]
fn main() -> ! {
    unsafe {
        embedded_alloc::init!(HEAP, HEAP_SIZE);
    }

    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    dp.EXINT
        .pcicr()
        .modify(|r, w| unsafe { w.bits(r.bits() | 1) }); // Enable PCINT0 group
    dp.EXINT
        .pcmsk0()
        .modify(|r, w| unsafe { w.bits(r.bits() | (1 << 4)) }); // Enable PCINT4 mask bit

    unsafe {
        avr_device::interrupt::enable();
    }

    let mut serial = arduino_hal::default_serial!(dp, pins, 115200);

    let mut serial_one = Usart::new(dp.USART1, pins.d19, pins.d18.into_output(), 115200.into());
    let rs485 = Rs485::new(pins.d2.into_output(), &mut serial_one);

    loop {
        let current = TOGGLED.load(core::sync::atomic::Ordering::SeqCst);
        ufmt::uwriteln!(&mut serial, "toggled?: {}", current);
        continue;

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
