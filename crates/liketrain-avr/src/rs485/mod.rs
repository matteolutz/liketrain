use embedded_hal::digital::OutputPin;
use liketrain_hardware::serial::{SerialError, SerialInterface};

#[derive(Debug)]
pub enum Rs485Error<E: core::fmt::Debug> {
    PinError,
    SerialError(SerialError<E>),
}

impl<E> From<SerialError<E>> for Rs485Error<E>
where
    E: core::fmt::Debug,
{
    fn from(value: SerialError<E>) -> Self {
        Self::SerialError(value)
    }
}

pub struct Rs485<'a, R, E: core::fmt::Debug> {
    re_de_pin: R,
    serial: &'a mut dyn SerialInterface<Error = E>,
}

impl<'a, R, E> Rs485<'a, R, E>
where
    R: OutputPin,
    E: core::fmt::Debug,
{
    pub fn new(re_de_pin: R, serial: &'a mut dyn SerialInterface<Error = E>) -> Self {
        Rs485 { re_de_pin, serial }
    }

    fn transmit(&mut self) -> Result<(), Rs485Error<E>> {
        self.re_de_pin.set_high().map_err(|_| Rs485Error::PinError)
    }

    fn receive(&mut self) -> Result<(), Rs485Error<E>> {
        self.re_de_pin.set_low().map_err(|_| Rs485Error::PinError)
    }
}

impl<'a, R, E> SerialInterface for Rs485<'a, R, E>
where
    R: OutputPin,
    E: core::fmt::Debug,
{
    type Error = Rs485Error<E>;

    fn write_byte(
        &mut self,
        byte: u8,
    ) -> Result<(), liketrain_hardware::serial::SerialError<Self::Error>> {
        self.transmit()?;
        self.serial
            .write_byte(byte)
            .map_err(Rs485Error::SerialError)?;
        Ok(())
    }

    fn write_bytes(
        &mut self,
        bytes: &[u8],
    ) -> Result<usize, liketrain_hardware::serial::SerialError<Self::Error>> {
        self.transmit()?;
        let bytes_read = self
            .serial
            .write_bytes(bytes)
            .map_err(|e| Rs485Error::SerialError(e))?;
        Ok(bytes_read)
    }

    fn read_max_bytes(
        &mut self,
        bytes: &mut [u8],
    ) -> Result<usize, liketrain_hardware::serial::SerialError<Self::Error>> {
        self.receive()?;
        let bytes_read = self
            .serial
            .read_max_bytes(bytes)
            .map_err(|e| Rs485Error::SerialError(e))?;
        Ok(bytes_read)
    }

    fn flush(&mut self) -> Result<(), liketrain_hardware::serial::SerialError<Self::Error>> {
        self.transmit()?;
        self.serial
            .flush()
            .map_err(|e| Rs485Error::SerialError(e))?;
        Ok(())
    }
}
