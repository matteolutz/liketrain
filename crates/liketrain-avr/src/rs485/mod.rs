use arduino_hal::{Usart, hal::Atmega, usart::UsartOps};
use embedded_hal::digital::OutputPin;

use crate::serial::UsartExt;

#[derive(Debug)]
pub enum Rs485Error {
    PinError,
    SerialError,
}

pub struct Rs485<'a, R, U> {
    re_de_pin: R,
    serial: &'a mut U,
}

impl<'a, R, USART, RX, TX> Rs485<'a, R, Usart<USART, RX, TX>>
where
    R: OutputPin,
    USART: UsartOps<Atmega, RX, TX>,
{
    pub fn new(re_de_pin: R, serial: &'a mut Usart<USART, RX, TX>) -> Self {
        Rs485 { re_de_pin, serial }
    }

    fn transmit(&mut self) -> Result<(), Rs485Error> {
        self.re_de_pin.set_high().map_err(|_| Rs485Error::PinError)
    }

    fn receive(&mut self) -> Result<(), Rs485Error> {
        self.re_de_pin.set_low().map_err(|_| Rs485Error::PinError)
    }
}

impl<'a, R, USART, RX, TX> UsartExt for Rs485<'a, R, Usart<USART, RX, TX>>
where
    R: OutputPin,
    USART: UsartOps<Atmega, RX, TX>,
{
    type Error = Rs485Error;

    fn write_struct<T>(&mut self, struct_data: &T) -> Result<(), Self::Error> {
        self.transmit()?;
        self.serial
            .write_struct(struct_data)
            .map_err(|_| Rs485Error::SerialError)
    }

    fn read_struct<T>(&mut self) -> Result<T, Self::Error> {
        self.receive()?;
        self.serial
            .read_struct()
            .map_err(|_| Rs485Error::SerialError)
    }

    fn try_read_struct<T>(&mut self) -> Result<T, Self::Error> {
        self.receive()?;
        self.serial
            .try_read_struct()
            .map_err(|_| Rs485Error::SerialError)
    }

    fn write_event(
        &mut self,
        event: liketrain_hardware::event::HardwareEvent,
    ) -> Result<(), Self::Error> {
        self.transmit()?;
        self.serial
            .write_event(event)
            .map_err(|_| Rs485Error::SerialError)
    }

    fn read_command(
        &mut self,
    ) -> Result<liketrain_hardware::command::HardwareCommand, Self::Error> {
        self.receive()?;
        self.serial
            .read_command()
            .map_err(|_| Rs485Error::SerialError)
    }

    fn try_read_command(
        &mut self,
    ) -> Result<liketrain_hardware::command::HardwareCommand, Self::Error> {
        self.receive()?;
        self.serial
            .try_read_command()
            .map_err(|_| Rs485Error::SerialError)
    }
}
