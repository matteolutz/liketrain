#[cfg(test)]
mod tests;

mod ext;
use std::io;

pub use ext::*;
use liketrain_hardware::serial::SerialInterface;

pub struct SerialportSerialInterface(Box<dyn serialport::SerialPort>);

impl From<Box<dyn serialport::SerialPort>> for SerialportSerialInterface {
    fn from(port: Box<dyn serialport::SerialPort>) -> Self {
        SerialportSerialInterface(port)
    }
}

impl SerialInterface for SerialportSerialInterface {
    type Error = std::io::Error;

    fn write_byte(
        &mut self,
        byte: u8,
    ) -> Result<(), liketrain_hardware::serial::SerialError<Self::Error>> {
        let bytes_written = self.0.write(&[byte])?;

        if bytes_written != 1 {
            Err(liketrain_hardware::serial::SerialError::FailedToWrite)
        } else {
            Ok(())
        }
    }

    fn write_bytes(
        &mut self,
        bytes: &[u8],
    ) -> Result<usize, liketrain_hardware::serial::SerialError<Self::Error>> {
        let bytes_written = self.0.write(bytes)?;
        Ok(bytes_written)
    }

    fn read_max_bytes(
        &mut self,
        bytes: &mut [u8],
    ) -> Result<usize, liketrain_hardware::serial::SerialError<Self::Error>> {
        let bytes_read = match self.0.read(bytes) {
            Ok(n) => n,
            // ignore timeout errors
            Err(err) if err.kind() == io::ErrorKind::TimedOut => 0,
            // fail on other errors
            err => err?,
        };

        Ok(bytes_read)
    }

    fn flush(&mut self) -> Result<(), liketrain_hardware::serial::SerialError<Self::Error>> {
        self.0.flush()?;
        Ok(())
    }
}
