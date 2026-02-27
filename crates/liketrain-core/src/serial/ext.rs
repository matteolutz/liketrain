use std::io;

use liketrain_hardware::{
    SERIAL_START_BYTE,
    command::{HardwareCommand, avr::HardwareCommandStruct},
    event::{HardwareEvent, avr::HardwareEventStruct},
};
use serialport::SerialPort;

pub trait SerialExt {
    /// Write a struct over the USART.
    fn write_struct<T>(&mut self, struct_data: &T) -> io::Result<()>;

    /// Try to read a struct from serial. This won't block to wait for the start byte, but fail if the start byte is not received.
    fn try_read_struct_from_stream<T>(&mut self, stream: &mut Vec<u8>) -> io::Result<Option<T>>;

    /// Write an event over the USART.
    fn write_command(&mut self, event: HardwareCommand) -> io::Result<()>;

    /// Try to read a command from the USART. This won't block to wait for the start byte, but fail if the start byte is not received.
    fn try_read_event_from_stream(
        &mut self,
        stream: &mut Vec<u8>,
    ) -> io::Result<Option<HardwareEvent>>;

    fn read_debug_message(&mut self, len: usize) -> io::Result<String>;
}

impl SerialExt for Box<dyn SerialPort> {
    fn write_struct<T>(&mut self, struct_data: &T) -> io::Result<()> {
        let size = core::mem::size_of::<T>();

        let bytes =
            unsafe { std::slice::from_raw_parts((struct_data as *const T) as *const u8, size) };

        let size = size as u16;
        let [size_low, size_high] = size.to_le_bytes();

        self.write_all(&[SERIAL_START_BYTE, size_low, size_high])?;

        self.write_all(bytes)?;

        let checksum = bytes
            .iter()
            .fold(0, |acc: u8, &byte| acc.wrapping_add(byte));
        self.write_all(&[checksum])?;

        self.flush()?;

        Ok(())
    }

    fn try_read_struct_from_stream<T>(&mut self, stream: &mut Vec<u8>) -> io::Result<Option<T>> {
        loop {
            if stream.len() < 1 {
                return Ok(None);
            }

            // look for start byte
            if stream[0] != SERIAL_START_BYTE {
                // skip invalid byte
                stream.remove(0);
                continue;
            }

            // needs at least 4 bytes (size + checksum)
            if stream.len() < 4 {
                return Ok(None);
            }

            let size = u16::from_le_bytes([stream[1], stream[2]]) as usize;

            if stream.len() < 3 + size + 1 {
                // the full message hasn't arrived yet
                return Ok(None);
            }

            let payload = &stream[3..3 + size];
            let checksum = stream[3 + size];

            let calculated_checksum = payload
                .iter()
                .fold(0, |acc: u8, &byte| acc.wrapping_add(byte));

            if checksum != calculated_checksum {
                // invalid checksum -> skip the start byte and resync
                stream.remove(0);
                continue;
            }

            if size != std::mem::size_of::<T>() {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid size"));
            }

            let value = unsafe { std::ptr::read(payload.as_ptr() as *const T) };

            stream.drain(0..(3 + size + 1));

            return Ok(Some(value));
        }
    }

    fn write_command(&mut self, command: HardwareCommand) -> io::Result<()> {
        let command_struct: HardwareCommandStruct = command.into();
        self.write_struct(&command_struct)
    }

    fn try_read_event_from_stream(
        &mut self,
        stream: &mut Vec<u8>,
    ) -> io::Result<Option<HardwareEvent>> {
        let event_struct: Option<HardwareEventStruct> = self.try_read_struct_from_stream(stream)?;
        Ok(event_struct.map(|o| o.into()))
    }

    fn read_debug_message(&mut self, len: usize) -> io::Result<String> {
        let mut start_byte_buf = [0_u8];
        loop {
            match self.read_exact(&mut start_byte_buf) {
                Ok(()) => {}
                Err(err) if err.kind() == io::ErrorKind::TimedOut => continue,
                Err(err) => return Err(err),
            };

            if start_byte_buf[0] == SERIAL_START_BYTE {
                break;
            }
        }

        let mut message_buf = vec![0_u8; len];
        self.read_exact(&mut message_buf)?;

        let str = String::from_utf8(message_buf)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))?;

        Ok(str)
    }
}
