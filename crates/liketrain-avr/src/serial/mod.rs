use core::mem::MaybeUninit;

use arduino_hal::{Usart, hal::Atmega, usart::UsartOps};
use liketrain_hardware::{
    SERIAL_START_BYTE,
    command::{HardwareCommand, avr::HardwareCommandStruct},
    event::{HardwareEvent, avr::HardwareEventStruct},
};

pub trait UsartExt {
    /// Write a struct over the USART.
    fn write_struct<T>(&mut self, struct_data: &T);

    /// Read a struct from the USART. This will block until the start byte is received.
    fn read_struct<T>(&mut self) -> Result<T, ()>;

    /// Try to read a struct from the USART. This will not block, but fail if the start byte is not received.
    fn try_read_struct<T>(&mut self) -> Result<T, ()>;

    /// Write an event over the USART.
    fn write_event(&mut self, event: HardwareEvent);

    /// Read a command from the USART. This will block until the start byte is received.
    fn read_command(&mut self) -> Result<HardwareCommand, ()>;

    /// Try to read a command from the USART. This will not block, but fail if the start byte is not received.
    fn try_read_command(&mut self) -> Result<HardwareCommand, ()>;
}

impl<USART: UsartOps<Atmega, RX, TX>, RX, TX> UsartExt for Usart<USART, RX, TX> {
    fn write_struct<T>(&mut self, struct_data: &T) {
        let size = core::mem::size_of::<T>();

        let bytes = unsafe {
            core::slice::from_raw_parts(
                (struct_data as *const T) as *const u8,
                core::mem::size_of::<T>(),
            )
        };

        self.write_byte(SERIAL_START_BYTE);
        let size = size as u16;

        let [low, high] = size.to_le_bytes();
        self.write_byte(low);
        self.write_byte(high);

        for byte in bytes {
            self.write_byte(*byte);
        }

        let checksum = bytes.iter().fold(0u8, |acc, &byte| acc.wrapping_add(byte));
        self.write_byte(checksum);

        self.flush();
    }

    fn read_struct<T>(&mut self) -> Result<T, ()> {
        while self.read_byte() != SERIAL_START_BYTE {
            // wait for start byte
        }

        let size_low = self.read_byte();
        let size_high = self.read_byte();
        let size = u16::from_le_bytes([size_low, size_high]);
        let size = size as usize;

        if size != core::mem::size_of::<T>() {
            return Err(());
        }

        let mut buffer = MaybeUninit::<T>::uninit();
        let buf_ptr = buffer.as_mut_ptr() as *mut u8;

        for i in 0..size {
            let byte = self.read_byte();
            unsafe { *buf_ptr.add(i) = byte };
        }

        let bytes = unsafe { core::slice::from_raw_parts(buf_ptr, size) };

        let checksum = self.read_byte();
        let calculated_checksum = bytes.iter().fold(0u8, |acc, &byte| acc.wrapping_add(byte));
        if checksum != calculated_checksum {
            return Err(());
        }

        let value = unsafe { buffer.assume_init() };
        Ok(value)
    }

    fn try_read_struct<T>(&mut self) -> Result<T, ()> {
        if self.read_byte() != SERIAL_START_BYTE {
            return Err(());
        }

        let size_low = self.read_byte();
        let size_high = self.read_byte();
        let size = u16::from_le_bytes([size_low, size_high]);
        let size = size as usize;

        if size != core::mem::size_of::<T>() {
            return Err(());
        }

        let mut buffer = MaybeUninit::<T>::uninit();
        let buf_ptr = buffer.as_mut_ptr() as *mut u8;

        for i in 0..size {
            let byte = self.read_byte();
            unsafe { *buf_ptr.add(i) = byte };
        }

        let bytes = unsafe { core::slice::from_raw_parts(buf_ptr, size) };

        let checksum = self.read_byte();
        let calculated_checksum = bytes.iter().fold(0u8, |acc, &byte| acc.wrapping_add(byte));
        if checksum != calculated_checksum {
            return Err(());
        }

        let value = unsafe { buffer.assume_init() };
        Ok(value)
    }

    fn write_event(&mut self, event: HardwareEvent) {
        let event: HardwareEventStruct = event.into();
        self.write_struct(&event);
    }

    fn read_command(&mut self) -> Result<HardwareCommand, ()> {
        let command_struct: HardwareCommandStruct = self.read_struct()?;
        Ok(command_struct.into())
    }

    fn try_read_command(&mut self) -> Result<HardwareCommand, ()> {
        let command_struct: HardwareCommandStruct = self.try_read_struct()?;
        Ok(command_struct.into())
    }
}
