use std::{
    sync::{LazyLock, Mutex},
    time::Duration,
};

use liketrain_hardware::{
    command::HardwareCommand,
    event::{HardwareEvent, HardwareSectionPower, avr::HardwareEventStruct},
};

use crate::serial::ext::SerialExt;

// prevent the tests from running in parallel
static SERIAL_LOCK: LazyLock<Mutex<()>> = LazyLock::new(Mutex::default);

#[ignore]
#[test]
fn test_serial_list() {
    let ports = serialport::available_ports().unwrap();
    for p in ports {
        println!("Port: {}", p.port_name);
    }
}

/*
#[ignore]
#[test]
fn test_serial_mega_ping() {
    let _lock = SERIAL_LOCK.lock().unwrap();

    let port = "COM3";

    let slave_id = 0;
    let seq = 69;

    let mut port = serialport::new(port, 115200)
        .timeout(Duration::from_secs(10))
        .open()
        .unwrap();

    println!("sending command");
    port.write_command(HardwareCommand::Ping { slave_id, seq })
        .unwrap();

    println!(
        "hardware event struct size: {}",
        size_of::<HardwareEventStruct>()
    );
    let response = port.read_event().unwrap();
    println!("Got response: {:?}", response);

    match response {
        HardwareEvent::Pong {
            slave_id: res_slave_id,
            seq: res_seq,
        } => {
            assert_eq!(slave_id, res_slave_id);
            assert_eq!(seq, res_seq);
        }
        _ => assert!(false),
    }
}

#[ignore]
#[test]
fn test_serial_mega_sim() {
    let _lock = SERIAL_LOCK.lock().unwrap();

    let port = "COM3";

    let mut port = serialport::new(port, 115200)
        .timeout(Duration::from_secs(10))
        .open()
        .unwrap();

    port.write_command(HardwareCommand::SetSectionPower {
        section_id: 24,
        power: HardwareSectionPower::Half,
    })
    .unwrap();

    loop {
        let event = port.read_event().unwrap();
        println!("got {:?}", event);
    }
}
*/
