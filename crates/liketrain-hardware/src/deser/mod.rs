#[cfg(feature = "avr")]
use alloc::vec::Vec;

mod variant;

pub struct DeserPayloadReader<'a>(&'a [u8]);

impl DeserPayloadReader<'_> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn buffer(&self) -> &[u8] {
        self.0
    }

    pub fn parse_u8<E: core::fmt::Debug>(&mut self) -> Result<u8, DeserError<E>> {
        let result = self
            .0
            .first()
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
}

impl DeserPayloadWriter<'_> {
    pub fn buffer(&mut self) -> &mut Vec<u8> {
        self.buffer
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
    type Variant: Copy + Clone + TryFrom<u8> + Into<u8>;

    type Error: core::fmt::Debug;

    /// Get the variant of this message
    fn variant(&self) -> Self::Variant;

    /// Deserialize the payload.
    /// It is guaranteed that `payload_buffer` is at exactly `payload_size` bytes long.
    fn deser_deserialize(
        variant: Self::Variant,
        buffer: DeserPayloadReader,
    ) -> Result<Self, DeserError<Self::Error>>;

    /// Serialize the message into a buffer.
    fn deser_serialize(&self, buffer: DeserPayloadWriter) -> Result<(), DeserError<Self::Error>>;
}

pub trait DeserHelper<T: Deser>: Sized {
    /// Serialize the message into a Vec<u8>
    fn serialize(&self) -> Result<Vec<u8>, DeserError<T::Error>> {
        let mut buffer = Vec::new();
        self.serialize_into(&mut buffer)?;
        Ok(buffer)
    }

    /// Serialize the message into a given buffer.
    fn serialize_into(&self, buffer: &mut Vec<u8>) -> Result<(), DeserError<T::Error>>;

    /// Deserialize the message from a buffer.
    fn deserialize(buffer: &[u8]) -> Result<Self, DeserError<T::Error>>;
}

impl<T: Deser> DeserHelper<T> for T {
    fn serialize_into(&self, buffer: &mut Vec<u8>) -> Result<(), DeserError<<T as Deser>::Error>> {
        let mut writer = DeserPayloadWriter { buffer };

        let variant = self.variant();

        let variant_byte: u8 = variant.into();
        writer.write_u8(variant_byte)?;

        self.deser_serialize(writer)?;

        Ok(())
    }

    fn deserialize(buffer: &[u8]) -> Result<Self, DeserError<<T as Deser>::Error>> {
        if buffer.is_empty() {
            return Err(DeserError::UnexpectedEndOfBuffer);
        }

        let variant = buffer[0];
        let variant: T::Variant = variant
            .try_into()
            .map_err(|_| DeserError::InvalidVariant(variant))?;

        let buffer = DeserPayloadReader(&buffer[1..]);

        Self::deser_deserialize(variant, buffer)
    }
}
