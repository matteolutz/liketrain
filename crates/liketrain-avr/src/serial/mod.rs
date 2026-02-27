use core::ops::{Deref, DerefMut, RangeBounds};

use alloc::vec::Vec;
use arduino_hal::{Usart, hal::Atmega, prelude::_embedded_hal_serial_Read, usart::UsartOps};
use liketrain_hardware::{
    SERIAL_START_BYTE,
    deser::{Deser, DeserHelper},
    serial::SerialInterface,
};

pub struct UsartInterface<USART: UsartOps<Atmega, RX, TX>, RX, TX>(Usart<USART, RX, TX>);

impl<USART: UsartOps<Atmega, RX, TX>, RX, TX> Deref for UsartInterface<USART, RX, TX> {
    type Target = Usart<USART, RX, TX>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<USART: UsartOps<Atmega, RX, TX>, RX, TX> DerefMut for UsartInterface<USART, RX, TX> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<USART: UsartOps<Atmega, RX, TX>, RX, TX> From<Usart<USART, RX, TX>>
    for UsartInterface<USART, RX, TX>
{
    fn from(value: Usart<USART, RX, TX>) -> Self {
        Self(value)
    }
}

impl<USART: UsartOps<Atmega, RX, TX>, RX, TX> SerialInterface for UsartInterface<USART, RX, TX> {
    type Error = ();

    fn write_byte(
        &mut self,
        byte: u8,
    ) -> Result<(), liketrain_hardware::serial::SerialError<Self::Error>> {
        self.0.write_byte(byte);
        Ok(())
    }

    fn write_bytes(
        &mut self,
        bytes: &[u8],
    ) -> Result<usize, liketrain_hardware::serial::SerialError<Self::Error>> {
        for &byte in bytes.iter() {
            self.write_byte(byte)?;
        }

        Ok(bytes.len())
    }

    fn read_max_bytes(
        &mut self,
        bytes: &mut [u8],
    ) -> Result<usize, liketrain_hardware::serial::SerialError<Self::Error>> {
        for i in 0..bytes.len() {
            match self.0.read() {
                Ok(byte) => {
                    bytes[i] = byte;
                }
                Err(err) if err == nb::Error::WouldBlock => {
                    return Ok(i);
                }
                Err(_) => Err(())?,
            }
        }

        Ok(bytes.len())
    }

    fn flush(&mut self) -> Result<(), liketrain_hardware::serial::SerialError<Self::Error>> {
        self.0.flush();
        Ok(())
    }
}

pub struct UartStream<'a>(&'a mut Vec<u8>);

impl Deref for UartStream<'_> {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl UartStream<'_> {
    /// Shifts the first byte from the stream.
    pub fn shift(&mut self) -> Option<u8> {
        if self.0.is_empty() {
            None
        } else {
            Some(self.0.remove(0))
        }
    }

    pub fn drain<R>(&mut self, range: R)
    where
        R: RangeBounds<usize>,
    {
        self.0.drain(range);
    }
}

pub trait UsartExt {
    type Error;

    fn checksum(data: impl IntoIterator<Item = u8>) -> u8 {
        data.into_iter().fold(0, |acc, byte| acc.wrapping_add(byte))
    }

    fn write_deser<T: Deser>(&mut self, data: &T) -> Result<(), Self::Error>;
    fn try_read_deser_from_stream<T: Deser>(stream: UartStream) -> Result<Option<T>, Self::Error>;
    // fn read_deser<T: Deser>(&mut self) -> Result<T, Self::Error>;
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

        self.flush();

        Ok(())
    }

    fn try_read_deser_from_stream<T: Deser>(
        mut stream: UartStream,
    ) -> Result<Option<T>, Self::Error> {
        loop {
            if stream.len() < 1 {
                return Ok(None);
            }

            if stream[0] != SERIAL_START_BYTE {
                // remove invalid byte
                stream.shift();
                continue;
            }

            // needs at leaast 4 bytes (size)
            if stream.len() < 6 {
                return Ok(None);
            }

            let size = u32::from_le_bytes([stream[1], stream[2], stream[3], stream[4]]) as usize;

            if stream.len() < 5 + size + 1 {
                return Ok(None);
            }

            let payload = &stream[5..5 + size];
            let checksum = stream[5 + size];

            if Self::checksum(payload.iter().copied()) != checksum {
                // remove invalid payload
                stream.shift();
                continue;
            }

            let data = T::deserialize(payload).map_err(|_| ())?;

            stream.drain(0..(5 + size + 1));

            return Ok(Some(data));
        }
    }
}
