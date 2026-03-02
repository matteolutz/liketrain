use core::ops::{Deref, DerefMut};

use arduino_hal::{Usart, hal::Atmega, prelude::_embedded_hal_serial_Read, usart::UsartOps};
use liketrain_hardware::serial::SerialInterface;

mod deser;
pub use deser::*;

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
        for (i, byte) in bytes.iter_mut().enumerate() {
            match self.0.read() {
                Ok(recv_byte) => {
                    *byte = recv_byte;
                }
                Err(nb::Error::WouldBlock) => {
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
