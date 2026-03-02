use liketrain_hardware::{
    deser::Deser,
    serial::{DeserSerialExt, DeserSerialExtError, Serial, SerialError},
};

pub struct AvrTimeout<F>
where
    F: Fn() -> u32,
{
    pub timeout_ms: u32,
    pub millis_fn: F,
}

pub trait AvrTimeoutExt: Fn() -> u32 + Sized {
    fn timeout(self, timeout: u32) -> AvrTimeout<Self> {
        AvrTimeout {
            timeout_ms: timeout,
            millis_fn: self,
        }
    }
}

impl<F> AvrTimeoutExt for F where F: Fn() -> u32 {}

pub trait AvrDeserSerialExt<E: core::fmt::Debug> {
    /// Read a [`Deser`] value from the serial interface.
    /// Will block until a value is available.
    fn wait_for<D: Deser>(&mut self) -> Result<D, DeserSerialExtError<D::Error, E>> {
        self._wait_for_timeout::<D, fn() -> u32>(None)
    }

    /// Read a [`Deser`] value from the serial interface.
    /// Will block until a value is available or the timeout is reached.
    fn wait_for_timeout<D: Deser, F>(
        &mut self,
        millis: impl Into<AvrTimeout<F>>,
    ) -> Result<D, DeserSerialExtError<D::Error, E>>
    where
        F: Fn() -> u32,
    {
        self._wait_for_timeout::<D, F>(Some(millis.into()))
    }

    fn _wait_for_timeout<D: Deser, F>(
        &mut self,
        millis: Option<AvrTimeout<F>>,
    ) -> Result<D, DeserSerialExtError<D::Error, E>>
    where
        F: Fn() -> u32;
}

impl<'a, E: core::fmt::Debug> AvrDeserSerialExt<E> for Serial<'a, E> {
    fn _wait_for_timeout<D: Deser, F>(
        &mut self,
        millis: Option<AvrTimeout<F>>,
    ) -> Result<D, DeserSerialExtError<D::Error, E>>
    where
        F: Fn() -> u32,
    {
        let start = millis.as_ref().map(|m| (m.millis_fn)());

        loop {
            self.update()?;

            if let Some(value) = self.read()? {
                return Ok(value);
            }

            if let Some(millis) = millis.as_ref()
                && (millis.millis_fn)() > start.unwrap() + millis.timeout_ms
            {
                return Err(SerialError::Timeout.into());
            }
        }
    }
}
