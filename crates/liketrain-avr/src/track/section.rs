use alloc::{boxed::Box, vec::Vec};
use embedded_hal::digital::{InputPin, OutputPin};
use liketrain_hardware::event::{HardwareEvent, HardwareSectionPower, SectionEvent};

type BoxedOutputPin = Box<dyn OutputPin<Error = core::convert::Infallible>>;
type BoxedInputPin = Box<dyn InputPin<Error = core::convert::Infallible>>;

#[derive(Debug)]
pub enum SectionError {
    PinError,
}

pub struct SectionPins<
    PA: OutputPin<Error = core::convert::Infallible> + 'static,
    PB: OutputPin<Error = core::convert::Infallible> + 'static,
    PC: OutputPin<Error = core::convert::Infallible> + 'static,
    PD: OutputPin<Error = core::convert::Infallible> + 'static,
    TD: InputPin<Error = core::convert::Infallible> + 'static,
> {
    pub power_a: PA,
    pub power_b: PB,
    pub power_c: PC,
    pub power_d: PD,
    pub train_detection: TD,
}

pub struct BoxedSectionPins {
    pub power_a: BoxedOutputPin,
    pub power_b: BoxedOutputPin,
    pub power_c: BoxedOutputPin,
    pub power_d: BoxedOutputPin,
    pub train_detection: BoxedInputPin,
}

impl BoxedSectionPins {
    pub fn new<PA, PB, PC, PD, TD>(pins: SectionPins<PA, PB, PC, PD, TD>) -> Self
    where
        PA: OutputPin<Error = core::convert::Infallible> + 'static,
        PB: OutputPin<Error = core::convert::Infallible> + 'static,
        PC: OutputPin<Error = core::convert::Infallible> + 'static,
        PD: OutputPin<Error = core::convert::Infallible> + 'static,
        TD: InputPin<Error = core::convert::Infallible> + 'static,
    {
        Self {
            power_a: Box::new(pins.power_a),
            power_b: Box::new(pins.power_b),
            power_c: Box::new(pins.power_c),
            power_d: Box::new(pins.power_d),
            train_detection: Box::new(pins.train_detection),
        }
    }
}

impl<PA, PB, PC, PD, TD> From<SectionPins<PA, PB, PC, PD, TD>> for BoxedSectionPins
where
    PA: OutputPin<Error = core::convert::Infallible> + 'static,
    PB: OutputPin<Error = core::convert::Infallible> + 'static,
    PC: OutputPin<Error = core::convert::Infallible> + 'static,
    PD: OutputPin<Error = core::convert::Infallible> + 'static,
    TD: InputPin<Error = core::convert::Infallible> + 'static,
{
    fn from(pins: SectionPins<PA, PB, PC, PD, TD>) -> Self {
        Self::new(pins)
    }
}

pub struct SectionPowerRelais {
    /// Pin for relais with 25%
    power_a: BoxedOutputPin,
    /// Pin for relais with 50%
    power_b: BoxedOutputPin,
    /// Pin for relais with 75%
    power_c: BoxedOutputPin,
    /// Pin for relais with 100%
    power_d: BoxedOutputPin,

    current_power: HardwareSectionPower,
}

impl SectionPowerRelais {
    /// When switching between power levels (not from or to 0%), delay for 10ms
    const SWITCHING_DELAY: u32 = 10;

    pub fn new(
        mut power_a: BoxedOutputPin,
        mut power_b: BoxedOutputPin,
        mut power_c: BoxedOutputPin,
        mut power_d: BoxedOutputPin,
    ) -> Result<Self, SectionError> {
        power_a.set_low().map_err(|_| SectionError::PinError)?;
        power_b.set_low().map_err(|_| SectionError::PinError)?;
        power_c.set_low().map_err(|_| SectionError::PinError)?;
        power_d.set_low().map_err(|_| SectionError::PinError)?;

        Ok(Self {
            power_a,
            power_b,
            power_c,
            power_d,

            current_power: HardwareSectionPower::Off,
        })
    }

    fn set_power_relais_state(
        &mut self,
        power: HardwareSectionPower,
        state: bool,
    ) -> Result<(), SectionError> {
        match power {
            HardwareSectionPower::Off => unreachable!(),
            HardwareSectionPower::Quarter => self
                .power_a
                .set_state(state.into())
                .map_err(|_| SectionError::PinError),
            HardwareSectionPower::Half => self
                .power_b
                .set_state(state.into())
                .map_err(|_| SectionError::PinError),
            HardwareSectionPower::ThreeQuarters => self
                .power_c
                .set_state(state.into())
                .map_err(|_| SectionError::PinError),
            HardwareSectionPower::Full => self
                .power_d
                .set_state(state.into())
                .map_err(|_| SectionError::PinError),
        }
    }

    pub fn current_power(&self) -> HardwareSectionPower {
        self.current_power
    }

    pub fn set_power(&mut self, power: HardwareSectionPower) -> Result<(), SectionError> {
        if power == self.current_power {
            return Ok(());
        }

        let previous_power = self.current_power;
        self.current_power = power;

        if previous_power != HardwareSectionPower::Off {
            // we need to turn off the previous power relais
            self.set_power_relais_state(previous_power, false)?;

            // if we need to power up a different relais, delay
            if power != HardwareSectionPower::Off {
                arduino_hal::delay_ms(Self::SWITCHING_DELAY);
            }
        }

        if power != HardwareSectionPower::Off {
            self.set_power_relais_state(power, true)?;
        }

        Ok(())
    }
}

pub struct Section {
    section_id: u32,

    /// The 4-relais board for powering the section
    power_relais: SectionPowerRelais,
    /// The pin for detecting a train on the section
    train_detection: BoxedInputPin,

    is_occupied: bool,
}

pub trait SectionDelegate {
    fn section_id(&self) -> u32;

    fn is_occupied(&self) -> bool;

    fn current_power(&self) -> HardwareSectionPower;
    fn set_power(&mut self, power: HardwareSectionPower) -> Result<(), SectionError>;

    fn reset(&mut self) -> Result<(), SectionError> {
        self.set_power(HardwareSectionPower::Off)
    }

    fn update(&mut self, event_list: &mut Vec<HardwareEvent>) -> Result<(), SectionError>;
}

impl Section {
    pub fn new(
        section_id: u32,
        power_relais: SectionPowerRelais,
        train_detection: BoxedInputPin,
    ) -> Self {
        Section {
            section_id,
            power_relais,
            train_detection,
            is_occupied: false,
        }
    }

    pub fn from_pins(section_id: u32, pins: impl Into<BoxedSectionPins>) -> Self {
        let pins = pins.into();

        Section::new(
            section_id,
            SectionPowerRelais::new(pins.power_a, pins.power_b, pins.power_c, pins.power_d)
                .unwrap(),
            pins.train_detection,
        )
    }
}

impl SectionDelegate for Section {
    fn section_id(&self) -> u32 {
        self.section_id
    }

    fn is_occupied(&self) -> bool {
        self.is_occupied
    }

    fn current_power(&self) -> HardwareSectionPower {
        self.power_relais.current_power()
    }

    fn set_power(&mut self, power: HardwareSectionPower) -> Result<(), SectionError> {
        self.power_relais.set_power(power)
    }

    fn update(&mut self, event_list: &mut Vec<HardwareEvent>) -> Result<(), SectionError> {
        let is_occupied = self
            .train_detection
            .is_high()
            .map_err(|_| SectionError::PinError)?;

        if is_occupied != self.is_occupied {
            self.is_occupied = is_occupied;

            if is_occupied {
                event_list.push(HardwareEvent::SectionEvent(SectionEvent::occupied(
                    self.section_id,
                )));
            } else {
                event_list.push(HardwareEvent::SectionEvent(SectionEvent::freed(
                    self.section_id,
                )));
            }
        }

        Ok(())
    }
}
