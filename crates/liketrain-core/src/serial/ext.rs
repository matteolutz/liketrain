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

    /// Read a struct from the USART. This will block until the entire struct is received.
    fn read_struct<T>(&mut self) -> io::Result<T>;

    /// Try to read a struct from serial. This won't block to wait for the start byte, but fail if the start byte is not received.
    fn try_read_struct<T>(&mut self) -> io::Result<T>;

    /// Write an event over the USART.
    fn write_command(&mut self, event: HardwareCommand) -> io::Result<()>;

    /// Read a command from the USART. This will block until the entire command is received.
    fn read_event(&mut self) -> io::Result<HardwareEvent>;

    /// Try to read a command from the USART. This won't block to wait for the start byte, but fail if the start byte is not received.
    fn try_read_event(&mut self) -> io::Result<HardwareEvent>;
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

    fn read_struct<T>(&mut self) -> io::Result<T> {
        let mut start_byte_buf = [0_u8];
        loop {
            self.read_exact(&mut start_byte_buf)?;
            if start_byte_buf[0] == SERIAL_START_BYTE {
                break;
            }
        }

        let mut size_buf = [0_u8; 2];
        self.read_exact(&mut size_buf)?;
        let size = u16::from_le_bytes(size_buf);
        let size = size as usize;

        if size != std::mem::size_of::<T>() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid size"));
        }

        let mut buffer = vec![0; size];

        self.read_exact(&mut buffer)?;

        let mut checksum_buf = [0_u8; 1];
        self.read_exact(&mut checksum_buf)?;
        let checksum = checksum_buf[0];

        let calculated_checksum = buffer
            .iter()
            .fold(0, |acc: u8, &byte| acc.wrapping_add(byte));

        if checksum != calculated_checksum {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid checksum",
            ));
        }

        let value = unsafe { std::ptr::read(buffer.as_ptr() as *const T) };

        Ok(value)
    }

    fn try_read_struct<T>(&mut self) -> io::Result<T> {
        let mut start_byte_buf = [0_u8];
        self.read_exact(&mut start_byte_buf)?;

        if start_byte_buf[0] != SERIAL_START_BYTE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid start byte",
            ));
        }

        let mut size_buf = [0_u8; 2];
        self.read_exact(&mut size_buf)?;
        let size = u16::from_le_bytes(size_buf);
        let size = size as usize;

        if size != std::mem::size_of::<T>() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid size"));
        }

        let mut buffer = vec![0; size];

        self.read_exact(&mut buffer)?;

        let mut checksum_buf = [0_u8; 1];
        self.read_exact(&mut checksum_buf)?;
        let checksum = checksum_buf[0];

        let calculated_checksum = buffer
            .iter()
            .fold(0, |acc: u8, &byte| acc.wrapping_add(byte));

        if checksum != calculated_checksum {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid checksum",
            ));
        }

        let value = unsafe { std::ptr::read(buffer.as_ptr() as *const T) };

        Ok(value)
    }

    fn write_command(&mut self, command: HardwareCommand) -> io::Result<()> {
        let command_struct: HardwareCommandStruct = command.into();
        self.write_struct(&command_struct)
    }

    fn read_event(&mut self) -> io::Result<HardwareEvent> {
        let event_struct: HardwareEventStruct = self.read_struct()?;
        Ok(event_struct.into())
    }

    fn try_read_event(&mut self) -> io::Result<HardwareEvent> {
        let event_struct: HardwareEventStruct = self.try_read_struct()?;
        Ok(event_struct.into())
    }
}
