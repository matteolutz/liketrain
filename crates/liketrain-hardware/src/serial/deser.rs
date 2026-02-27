use crate::{
    deser::{Deser, DeserError, DeserHelper},
    serial::{Serial, SerialError},
};

#[derive(Debug)]
pub enum DeserSerialExtError<D: core::fmt::Debug, S: core::fmt::Debug> {
    DeserError(DeserError<D>),
    SerialError(SerialError<S>),
}

impl<D: core::fmt::Debug, S: core::fmt::Debug> From<DeserError<D>> for DeserSerialExtError<D, S> {
    fn from(error: DeserError<D>) -> Self {
        DeserSerialExtError::DeserError(error)
    }
}

impl<D: core::fmt::Debug, S: core::fmt::Debug> From<SerialError<S>> for DeserSerialExtError<D, S> {
    fn from(error: SerialError<S>) -> Self {
        DeserSerialExtError::SerialError(error)
    }
}

pub trait DeserSerialExt<E: core::fmt::Debug> {
    const DESER_SERIAL_START_BYTE: u8 = 0xAA;

    fn checksum(data: impl IntoIterator<Item = u8>) -> u8 {
        data.into_iter().fold(0, |acc, byte| acc.wrapping_add(byte))
    }

    fn write<T: Deser>(&mut self, data: &T) -> Result<(), DeserSerialExtError<T::Error, E>>;
    fn read<T: Deser>(&mut self) -> Result<Option<T>, DeserSerialExtError<T::Error, E>>;
}

impl<'a, E> DeserSerialExt<E> for Serial<'a, E>
where
    E: core::fmt::Debug,
{
    fn write<T: Deser>(&mut self, data: &T) -> Result<(), DeserSerialExtError<T::Error, E>> {
        let data = data.serialize()?;

        let data_size: u32 = data.len() as u32;

        self.write_byte(Self::DESER_SERIAL_START_BYTE)?;

        for byte in data_size.to_le_bytes() {
            self.write_byte(byte)?;
        }

        for &byte in &data {
            self.write_byte(byte)?;
        }

        let checksum = Self::checksum(data);
        self.write_byte(checksum)?;

        self.flush()?;

        Ok(())
    }

    fn read<T: Deser>(&mut self) -> Result<Option<T>, DeserSerialExtError<T::Error, E>> {
        loop {
            if self.stream().is_empty() {
                return Ok(None);
            }

            if self.stream()[0] != Self::DESER_SERIAL_START_BYTE {
                // remove invalid byte
                self.stream_mut().shift();
                continue;
            }

            // needs at least 1 byte (start byte) + 4 bytes (size) + 1 byte (checksum)
            if self.stream().len() < 6 {
                return Ok(None);
            }

            let size = u32::from_le_bytes([
                self.stream()[1],
                self.stream()[2],
                self.stream()[3],
                self.stream()[4],
            ]) as usize;

            if self.stream().len() < 5 + size + 1 {
                return Ok(None);
            }

            let payload = &self.stream()[5..5 + size];
            let checksum = self.stream()[5 + size];

            if Self::checksum(payload.iter().copied()) != checksum {
                // remove invalid payload
                self.stream_mut().shift();
                continue;
            }

            let data = T::deserialize(payload)?;

            self.stream_mut().drain(0..(5 + size + 1));

            return Ok(Some(data));
        }
    }
}
