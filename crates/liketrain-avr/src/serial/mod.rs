use core::mem::MaybeUninit;

use arduino_hal::{
    Usart,
    hal::Atmega,
    prelude::{_embedded_hal_serial_Read, _embedded_hal_serial_Write},
    usart::UsartOps,
};
use liketrain_hardware::{
    SERIAL_START_BYTE,
    command::{HardwareCommand, avr::HardwareCommandStruct},
    deser::{Deser, DeserHelper},
    event::{HardwareEvent, avr::HardwareEventStruct},
};
use ufmt::uWrite;

use crate::mode::{SlaveCommand, SlaveResponse};

pub trait UsartExt {
    type Error;

    fn checksum(data: impl IntoIterator<Item = u8>) -> u8 {
        data.into_iter().fold(0, |acc, byte| acc.wrapping_add(byte))
    }

    fn write_deser<T: Deser>(&mut self, data: &T) -> Result<(), Self::Error>;
    fn read_deser<T: Deser>(&mut self) -> Result<T, Self::Error>;

    /// Write a struct over the USART.
    fn write_struct<T>(&mut self, struct_data: &T) -> Result<(), Self::Error>;

    /// Read a struct from the USART. This will block until the start byte is received.
    fn read_struct<T>(&mut self) -> Result<T, Self::Error>;

    /// Try to read a struct from the USART. This will not block, but fail if the start byte is not received.
    fn try_read_struct<T>(&mut self) -> Result<T, Self::Error>;

    /// Write an event over the USART.
    fn write_event(&mut self, event: HardwareEvent) -> Result<(), Self::Error> {
        let event: HardwareEventStruct = event.into();
        self.write_struct(&event)?;
        Ok(())
    }

    /// Write multiple events over the USART.
    fn write_events(
        &mut self,
        events: impl IntoIterator<Item = HardwareEvent>,
    ) -> Result<(), Self::Error> {
        for event in events {
            self.write_event(event)?;
        }
        Ok(())
    }

    /// Read an event from the USART. This will block until the start byte is received.
    fn read_event(&mut self) -> Result<HardwareEvent, Self::Error> {
        let event_struct: HardwareEventStruct = self.read_struct()?;
        Ok(event_struct.into())
    }

    /// Try to read an event from the USART. This will not block, but fail if the start byte is not received.
    fn try_read_event(&mut self) -> Result<HardwareEvent, Self::Error> {
        let event_struct: HardwareEventStruct = self.try_read_struct()?;
        Ok(event_struct.into())
    }

    /// Write a command to the USART.
    fn write_command(&mut self, command: HardwareCommand) -> Result<(), Self::Error> {
        let command_struct: HardwareCommandStruct = command.into();
        self.write_struct(&command_struct)
    }

    /// Write multiple commands to the USART.
    fn write_commands(
        &mut self,
        commands: impl IntoIterator<Item = HardwareCommand>,
    ) -> Result<(), Self::Error> {
        for command in commands {
            self.write_command(command)?;
        }
        Ok(())
    }

    /// Read a command from the USART. This will block until the start byte is received.
    fn read_command(&mut self) -> Result<HardwareCommand, Self::Error> {
        let command_struct: HardwareCommandStruct = self.read_struct()?;
        Ok(command_struct.into())
    }

    /// Try to read a command from the USART. This will not block, but fail if the start byte is not received.
    fn try_read_command(&mut self) -> Result<HardwareCommand, Self::Error> {
        let command_struct: HardwareCommandStruct = self.try_read_struct()?;
        Ok(command_struct.into())
    }

    /// Write a slave command to the USART.
    fn write_slave_command(&mut self, command: SlaveCommand) -> Result<(), Self::Error> {
        self.write_struct(&command)
    }

    /// Write multiple slave commands to the USART.
    fn write_slave_commands(
        &mut self,
        commands: impl IntoIterator<Item = SlaveCommand>,
    ) -> Result<(), Self::Error> {
        for command in commands {
            self.write_slave_command(command)?;
        }
        Ok(())
    }

    /// Read a slave command from the USART. This will block until the start byte is received.
    fn read_slave_command(&mut self) -> Result<SlaveCommand, Self::Error> {
        self.read_struct()
    }

    /// Try to read a slave command from the USART. This will not block.
    fn try_read_slave_command(&mut self) -> Result<SlaveCommand, Self::Error> {
        self.try_read_struct()
    }

    /// Write a slave response to the USART.
    fn write_slave_response(&mut self, response: SlaveResponse) -> Result<(), Self::Error> {
        self.write_struct(&response)
    }

    /// Write multiple slave responses to the USART.
    fn write_slave_responses(
        &mut self,
        responses: impl IntoIterator<Item = SlaveResponse>,
    ) -> Result<(), Self::Error> {
        for response in responses {
            self.write_slave_response(response)?;
        }
        Ok(())
    }

    /// Read a slave response from the USART. This will block until the start byte is received.
    fn read_slave_response(&mut self) -> Result<SlaveResponse, Self::Error> {
        self.read_struct()
    }

    /// Try to read a slave response from the USART. This will not block.
    fn try_read_slave_response(&mut self) -> Result<SlaveResponse, Self::Error> {
        self.try_read_struct()
    }

    fn write_debug_message(&mut self, message: &str) -> Result<(), Self::Error>;
}

impl<USART: UsartOps<Atmega, RX, TX>, RX, TX> UsartExt for Usart<USART, RX, TX> {
    type Error = ();

    fn write_deser<T: Deser>(&mut self, data: &T) -> Result<(), Self::Error> {
        let data = data.serialize().map_err(|_| ())?;
        let data_size: u32 = data.len() as u32;

        self.write_byte(SERIAL_START_BYTE);

        for byte in data_size.to_le_bytes() {
            self.write_byte(byte);
        }

        for &byte in &data {
            self.write_byte(byte);
        }

        let checksum = Self::checksum(data);
        self.write_byte(checksum);

        Ok(())
    }

    fn write_struct<T>(&mut self, struct_data: &T) -> Result<(), Self::Error> {
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

        Ok(())
    }

    fn read_struct<T>(&mut self) -> Result<T, Self::Error> {
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

    fn try_read_struct<T>(&mut self) -> Result<T, Self::Error> {
        match self.read() {
            Ok(byte) => {
                if byte != SERIAL_START_BYTE {
                    return Err(());
                }
            }
            Err(nb::Error::WouldBlock) => return Err(()),
            Err(_) => return Err(()),
        };

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

    fn write_debug_message(&mut self, message: &str) -> Result<(), Self::Error> {
        let bytes = message.as_bytes();

        /*
        self.write_event(HardwareEvent::DebugMessage {
            len: bytes.len() as u32,
        })?;*/

        self.write_byte(SERIAL_START_BYTE);
        for byte in bytes {
            self.write_byte(*byte);
        }

        self.flush();
        Ok(())
    }
}
