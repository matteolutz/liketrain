#[cfg(feature = "avr")]
use alloc::vec::Vec;

mod deser;
pub use deser::*;

const SERIAL_BUFFER_SIZE: usize = 128;

pub struct SerialStream(Vec<u8>);

impl SerialStream {
    pub fn shift(&mut self) -> Option<u8> {
        if self.is_empty() {
            None
        } else {
            Some(self.remove(0))
        }
    }
}

impl core::ops::Deref for SerialStream {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::ops::DerefMut for SerialStream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub enum SerialError<E: core::fmt::Debug> {
    FailedToWrite,

    Error(E),
}

impl<E: core::fmt::Debug> From<E> for SerialError<E> {
    fn from(error: E) -> Self {
        SerialError::Error(error)
    }
}

pub trait SerialInterface {
    type Error: core::fmt::Debug;

    /// Writes a single byte over the serial interface.
    fn write_byte(&mut self, byte: u8) -> Result<(), SerialError<Self::Error>>;

    /// Writes multiple bytes over the serial interface.
    /// Returns the number of bytes written.
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<usize, SerialError<Self::Error>>;

    fn print(&mut self, message: &str) -> Result<(), SerialError<Self::Error>> {
        self.write_bytes(message.as_bytes())?;
        Ok(())
    }

    /// Receives up to `bytes.len()` bytes from the serial interface.
    /// Returns the number of bytes received.
    /// When timeout occurs, this should return Ok(0)
    fn read_max_bytes(&mut self, bytes: &mut [u8]) -> Result<usize, SerialError<Self::Error>>;

    /// Flushes the output buffer.
    fn flush(&mut self) -> Result<(), SerialError<Self::Error>>;
}

pub struct Serial<'a, E: core::fmt::Debug> {
    interface: &'a mut dyn SerialInterface<Error = E>,

    input_buffer: [u8; SERIAL_BUFFER_SIZE],
    read_stream: SerialStream,
}

impl<'a, E> Serial<'a, E>
where
    E: core::fmt::Debug,
{
    pub fn new(interface: &'a mut dyn SerialInterface<Error = E>) -> Self {
        Self {
            interface,
            input_buffer: [0; SERIAL_BUFFER_SIZE],
            read_stream: SerialStream(Vec::new()),
        }
    }

    pub fn interface(&self) -> &dyn SerialInterface<Error = E> {
        self.interface
    }

    pub fn interface_mut(&mut self) -> &mut dyn SerialInterface<Error = E> {
        self.interface
    }

    pub fn stream(&self) -> &SerialStream {
        &self.read_stream
    }

    pub fn stream_mut(&mut self) -> &mut SerialStream {
        &mut self.read_stream
    }

    pub fn write_byte(&mut self, byte: u8) -> Result<(), SerialError<E>> {
        self.interface.write_byte(byte)
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<usize, SerialError<E>> {
        self.interface.write_bytes(bytes)
    }

    pub fn flush(&mut self) -> Result<(), SerialError<E>> {
        self.interface.flush()
    }

    /// Updates the serial interface.
    /// This means reading any available bytes from the interface and adding them to the read stream.
    pub fn update(&mut self) -> Result<(), SerialError<E>> {
        let bytes_read = self.interface.read_max_bytes(&mut self.input_buffer)?;
        self.read_stream
            .extend_from_slice(&self.input_buffer[..bytes_read]);

        Ok(())
    }
}
