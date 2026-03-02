use liketrain_hardware::{
    deser::{Deser, DeserHelper, DeserSize},
    deser_variant,
    event::HardwareEvent,
};

#[derive(Debug, Clone)]
pub enum SlaveResponse {
    EventCount { count: u32 },
    Event(HardwareEvent),
}

impl From<HardwareEvent> for SlaveResponse {
    fn from(event: HardwareEvent) -> Self {
        SlaveResponse::Event(event)
    }
}

deser_variant! {
    SlaveResponseType {
        EventCount = 0x0,
        Event = 0x10,
    }
}

impl Deser for SlaveResponse {
    type Variant = SlaveResponseType;

    type Error = ();

    fn payload_size(variant: Self::Variant) -> DeserSize {
        match variant {
            SlaveResponseType::EventCount => size_of::<u32>().into(),
            SlaveResponseType::Event => DeserSize::Irrelevant,
        }
    }

    fn variant(&self) -> Self::Variant {
        match self {
            SlaveResponse::EventCount { .. } => SlaveResponseType::EventCount,
            SlaveResponse::Event { .. } => SlaveResponseType::Event,
        }
    }

    fn deser_deserialize(
        variant: Self::Variant,
        _payload_size: u32,
        mut buffer: liketrain_hardware::deser::DeserPayloadReader,
    ) -> Result<Self, liketrain_hardware::deser::DeserError<Self::Error>> {
        match variant {
            SlaveResponseType::EventCount => {
                let count = buffer.parse_u32()?;
                Ok(SlaveResponse::EventCount { count })
            }
            SlaveResponseType::Event => {
                let event = HardwareEvent::deserialize(buffer.buffer())?;
                Ok(SlaveResponse::Event(event))
            }
        }
    }

    fn deser_serialize(
        &self,
        mut buffer: liketrain_hardware::deser::DeserPayloadWriter,
    ) -> Result<(), liketrain_hardware::deser::DeserError<Self::Error>> {
        match self {
            SlaveResponse::EventCount { count } => {
                buffer.write_u32(*count)?;
                Ok(())
            }
            SlaveResponse::Event(event) => {
                event.serialize_into(buffer.buffer())?;
                Ok(())
            }
        }
    }
}
