#[cfg(feature = "avr")]
use alloc::vec::Vec;

pub enum DeserSize {
    /// For this message type, the payload size is fixed
    Fixed(u32),

    /// For this message type, the payload size is variable.
    /// This means we need to read the size as the next u32 in the buffer.
    Variable,
}

impl From<usize> for DeserSize {
    fn from(value: usize) -> Self {
        DeserSize::Fixed(value as u32)
    }
}

pub struct DeserPayloadReader<'a>(&'a [u8]);

impl DeserPayloadReader<'_> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn parse_u8<E: core::fmt::Debug>(&mut self) -> Result<u8, DeserError<E>> {
        let result = self
            .0
            .get(0)
            .copied()
            .ok_or(DeserError::UnexpectedEndOfBuffer)?;
        self.0 = &self.0[1..];
        Ok(result)
    }

    pub fn parse_u16<E: core::fmt::Debug>(&mut self) -> Result<u16, DeserError<E>> {
        let result = u16::from_le_bytes([self.parse_u8()?, self.parse_u8()?]);
        Ok(result)
    }

    pub fn parse_u32<E: core::fmt::Debug>(&mut self) -> Result<u32, DeserError<E>> {
        let result = u32::from_le_bytes([
            self.parse_u8()?,
            self.parse_u8()?,
            self.parse_u8()?,
            self.parse_u8()?,
        ]);
        Ok(result)
    }

    pub fn parse<T, E: core::fmt::Debug>(&mut self) -> Result<T, DeserError<E>> {
        let size = core::mem::size_of::<T>();
        let mut buffer = core::mem::MaybeUninit::<T>::uninit();
        let buf_ptr = buffer.as_mut_ptr() as *mut u8;

        for i in 0..size {
            let byte = self.parse_u8()?;
            unsafe { *buf_ptr.add(i) = byte };
        }

        let value = unsafe { buffer.assume_init() };
        Ok(value)
    }
}

pub struct DeserPayloadWriter<'a> {
    buffer: &'a mut Vec<u8>,
    size_written: &'a mut bool,
}

impl DeserPayloadWriter<'_> {
    pub fn write_size<E: core::fmt::Debug>(&mut self, value: u32) -> Result<(), DeserError<E>> {
        if *self.size_written {
            return Err(DeserError::SizeWasAlreadyWritten);
        }

        self.write_u32(value)?;
        *self.size_written = true;

        Ok(())
    }

    pub fn write_u8<E: core::fmt::Debug>(&mut self, value: u8) -> Result<(), DeserError<E>> {
        self.buffer.push(value);
        Ok(())
    }

    pub fn write_u16<E: core::fmt::Debug>(&mut self, value: u16) -> Result<(), DeserError<E>> {
        self.write_u8(value as u8)?;
        self.write_u8((value >> 8) as u8)?;
        Ok(())
    }

    pub fn write_u32<E: core::fmt::Debug>(&mut self, value: u32) -> Result<(), DeserError<E>> {
        self.write_u16(value as u16)?;
        self.write_u16((value >> 16) as u16)?;
        Ok(())
    }

    pub fn write<T, E: core::fmt::Debug>(&mut self, value: &T) -> Result<(), DeserError<E>> {
        let size = core::mem::size_of::<T>();
        let buffer = value as *const T as *const u8;

        for i in 0..size {
            let byte = unsafe { *buffer.add(i) };
            self.write_u8(byte)?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum DeserError<E: core::fmt::Debug> {
    UnexpectedEndOfBuffer,

    SizeWasAlreadyWritten,
    SizeWasNotWritten,

    InvalidVariant(u8),

    Error(E),
}

impl<E: core::fmt::Debug> From<E> for DeserError<E> {
    fn from(value: E) -> Self {
        DeserError::Error(value)
    }
}

pub trait Deser: Sized {
    /// The variants enum of this message type
    type Variant: Copy + Clone + TryFrom<u8>;

    type Error: core::fmt::Debug;

    /// Get the size of the payload for a given variant
    fn payload_size(variant: Self::Variant) -> DeserSize;

    /// Get the variant of this message
    fn variant(&self) -> Self::Variant;

    /// Deserialize the payload.
    /// It is guaranteed that `payload_buffer` is at exactly `payload_size` bytes long.
    fn deser_deserialize(
        variant: Self::Variant,
        payload_size: u32,
        buffer: DeserPayloadReader,
    ) -> Result<Self, DeserError<Self::Error>>;

    /// Serialize the message into a buffer.
    fn deser_serialize(&self, buffer: DeserPayloadWriter) -> Result<(), DeserError<Self::Error>>;
}

pub trait DeserHelper<T: Deser>: Sized {
    /// Serialize the message into a buffer.
    fn serialize(&self) -> Result<Vec<u8>, DeserError<T::Error>>;

    /// Deserialize the message from a buffer.
    fn deserialize(buffer: &[u8]) -> Result<Self, DeserError<T::Error>>;
}

impl<T: Deser> DeserHelper<T> for T {
    fn serialize(&self) -> Result<Vec<u8>, DeserError<<T as Deser>::Error>> {
        let mut buffer = Vec::new();
        let mut size_written = false;

        let writer = DeserPayloadWriter {
            buffer: &mut buffer,
            size_written: &mut size_written,
        };

        let variant = self.variant();
        let size = Self::payload_size(variant);

        self.deser_serialize(writer)?;

        if matches!(size, DeserSize::Variable) && !size_written {
            return Err(DeserError::SizeWasNotWritten);
        }

        Ok(buffer)
    }

    fn deserialize(buffer: &[u8]) -> Result<Self, DeserError<<T as Deser>::Error>> {
        if buffer.is_empty() {
            return Err(DeserError::UnexpectedEndOfBuffer);
        }

        let variant = buffer[0];
        let variant: T::Variant = variant
            .try_into()
            .map_err(|_| DeserError::InvalidVariant(variant))?;

        let mut buffer = DeserPayloadReader(&buffer[1..]);

        let size = match T::payload_size(variant) {
            DeserSize::Fixed(size) => size,
            DeserSize::Variable => {
                if buffer.len() < 4 {
                    return Err(DeserError::UnexpectedEndOfBuffer);
                }

                buffer.parse_u32::<T::Error>().unwrap()
            }
        };

        Self::deser_deserialize(variant, size, buffer)
    }
}
