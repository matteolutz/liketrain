#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

extern crate alloc;

use core::cell::Cell;

use alloc::vec::Vec;
use arduino_hal::Usart;
use avr_device::interrupt::Mutex;
use embedded_alloc::LlffHeap;
use liketrain_hardware::{
    command::HardwareCommand,
    event::HardwareEvent,
    serial::{DeserSerialExt, Serial},
};
use panic_halt as _;

#[cfg(feature = "sim")]
use crate::sim::SimTrain;
use crate::{
    command::{CommandExecutionContext, CommandExt},
    mode::{LiketrainMode, SlaveCommand, SlaveId, SlaveResponse},
    rs485::Rs485,
    serial::UsartInterface,
    track::{Section, SectionDelegate, SectionPowerRelais},
};

mod command;
mod mode;
mod rs485;
mod serial;
mod track;

#[cfg(feature = "sim")]
mod sim;

const MODE: LiketrainMode = LiketrainMode::Master;

#[global_allocator]
static HEAP: LlffHeap = LlffHeap::empty();
const HEAP_SIZE: usize = 1024;

static MILLIS: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));

#[avr_device::interrupt(atmega2560)]
fn TIMER0_OVF() {
    avr_device::interrupt::free(|cs| {
        let counter = MILLIS.borrow(cs);
        counter.set(counter.get().wrapping_add(1));
    });
}

fn millis() -> u32 {
    avr_device::interrupt::free(|cs| {
        let counter = MILLIS.borrow(cs);
        counter.get()
    })
}

#[arduino_hal::entry]
fn main() -> ! {
    unsafe {
        embedded_alloc::init!(HEAP, HEAP_SIZE);
    }

    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let tc0 = dp.TC0;
    tc0.tccr0a().write(|w| unsafe { w.wgm0().bits(0) });
    tc0.tccr0b().write(|w| w.cs0().prescale_64());
    tc0.timsk0().write(|w| w.toie0().set_bit());

    unsafe {
        avr_device::interrupt::enable();
    }

    let mut builtin_in = pins.d13.into_output();

    let mut section_24 = Section::new(
        24,
        SectionPowerRelais::new(
            pins.d4.into_output(),
            pins.d5.into_output(),
            pins.d6.into_output(),
            pins.d7.into_output(),
        )
        .unwrap(),
        pins.d8,
    );

    let mut section_22 = Section::new(
        22,
        SectionPowerRelais::new(
            pins.d9.into_output(),
            pins.d10.into_output(),
            pins.d11.into_output(),
            pins.d12.into_output(),
        )
        .unwrap(),
        pins.d26,
    );

    let mut section_21 = Section::new(
        21,
        SectionPowerRelais::new(
            pins.d21.into_output(),
            pins.d22.into_output(),
            pins.d23.into_output(),
            pins.d24.into_output(),
        )
        .unwrap(),
        pins.d25,
    );

    let slave_ids: [SlaveId; 0] = [];
    let mut sections: [&mut dyn SectionDelegate; 3] =
        [&mut section_21, &mut section_22, &mut section_24];

    #[cfg(feature = "sim")]
    let mut sim_train: SimTrain<4> =
        SimTrain::new([(24, 5000), (22, 5000), (21, 5000), (24, 5000)]);

    let mut usb_serial = arduino_hal::default_serial!(dp, pins, 115200);
    let mut usb_serial_interface = UsartInterface::from(usb_serial);
    let mut usb_serial = Serial::new(&mut usb_serial_interface);

    let mut serial_one = Usart::new(dp.USART1, pins.d19, pins.d18.into_output(), 115200.into());
    let mut rs485 = Rs485::new(pins.d2.into_output(), &mut serial_one);

    let mut event_list = Vec::new();
    let mut slave_commands = Vec::new();
    let mut debug_messages = Vec::new();

    let mut execution_ctx = CommandExecutionContext {
        mode: MODE,
        event_list: &mut event_list,
        sections: &mut sections,
        debug_messages: &mut debug_messages,
    };

    loop {
        // receive incoming commands, if slave, send events when requested
        match MODE {
            LiketrainMode::Master => {
                // usb_serial.interface_mut().print("working");
                let _ = usb_serial.update();

                while let Some(command) = usb_serial.read::<HardwareCommand>().ok().flatten() {
                    let Ok(was_handled) = command.execute(&mut execution_ctx) else {
                        continue;
                    };

                    if !was_handled {
                        slave_commands.push(command);
                    }
                }
            }
            LiketrainMode::Slave { slave_id } => {
                /*
                while let Ok(slave_command) = rs485.try_read_slave_command() {
                    match slave_command {
                        SlaveCommand::Command(command) => {
                            let command: HardwareCommand = command.into();
                            let _ = command.execute(&mut execution_ctx);
                        }
                        SlaveCommand::EventPoll {
                            slave_id: poll_slave_id,
                        } if slave_id == poll_slave_id => {
                            let num_events = execution_ctx.event_list.len();

                            let _ = rs485.write_slave_response(SlaveResponse::EventCount {
                                count: num_events as u32,
                            });
                            let _ = rs485.write_slave_responses(
                                execution_ctx.event_list.drain(..).map(|evt| evt.into()),
                            );
                        }
                        _ => {}
                    }
                }
                */
            }
        }

        // send unhandled commands to slave bus
        if MODE.is_master() {
            // let _ = rs485.write_slave_commands(slave_commands.drain(..).map(|cmd| cmd.into()));
        }

        // update sim trains
        #[cfg(feature = "sim")]
        sim_train.update(&mut execution_ctx, millis());

        // update own sections
        for section in execution_ctx.sections.iter_mut() {
            let _ = section.update(execution_ctx.event_list);
        }

        // poll slaves
        if MODE.is_master() {
            /*
            for slave_id in &slave_ids {
                let _ = rs485.write_slave_command(SlaveCommand::EventPoll {
                    slave_id: *slave_id,
                });

                // block for the first response, this should be the event count
                let Ok(SlaveResponse::EventCount { count: event_count }) =
                    rs485.read_slave_response()
                else {
                    continue;
                };

                // recieve as many events as the slave said it had
                for _ in 0..event_count {
                    let Ok(SlaveResponse::Event(event)) = rs485.read_slave_response() else {
                        break;
                    };

                    execution_ctx.event_list.push(event.into());
                }
            }
            ´*/
        }

        // send events back to master or host
        match MODE {
            LiketrainMode::Master => {
                for event in execution_ctx.event_list.drain(..) {
                    let _ = usb_serial.write(&event);
                }

                for message in execution_ctx.debug_messages.drain(..) {
                    let _ = usb_serial.write(&HardwareEvent::DebugMessage { message });
                }
            }
            LiketrainMode::Slave { .. } => {
                // do nothing, wait for a master poll, then drain the event list
            }
        }
    }
}
