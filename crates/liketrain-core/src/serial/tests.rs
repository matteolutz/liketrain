use std::time::Duration;

use liketrain_hardware::{
    command::HardwareCommand,
    event::{HardwareEvent, avr::HardwareEventStruct},
};

use crate::serial::ext::SerialExt;

#[ignore]
#[test]
fn test_serial_list() {
    let ports = serialport::available_ports().unwrap();
    for p in ports {
        println!("Port: {}", p.port_name);
    }
}

#[ignore]
#[test]
fn test_serial_mega() {
    let port = "COM3";
    let pong_id = 69;

    let mut port = serialport::new(port, 115200)
        .timeout(Duration::from_secs(10))
        .open()
        .unwrap();

    println!("sending command");
    port.write_command(HardwareCommand::Ping(pong_id)).unwrap();

    println!(
        "hardware event struct size: {}",
        size_of::<HardwareEventStruct>()
    );
    let response = port.read_event().unwrap();
    println!("Got response: {:?}", response);

    match response {
        HardwareEvent::Pong(res_pong_id) => assert_eq!(res_pong_id, pong_id),
        _ => assert!(false),
    }
}
