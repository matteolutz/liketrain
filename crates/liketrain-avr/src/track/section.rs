use alloc::vec::Vec;
use embedded_hal::digital::{InputPin, OutputPin};
use liketrain_hardware::event::{HardwareEvent, HardwareSectionPower, SectionEvent};

#[derive(Debug)]
pub enum SectionError {
    PinError,
}

pub struct SectionPowerRelais<A, B, C, D> {
    /// Pin for relais with 25%
    power_a: A,
    /// Pin for relais with 50%
    power_b: B,
    /// Pin for relais with 75%
    power_c: C,
    /// Pin for relais with 100%
    power_d: D,

    current_power: HardwareSectionPower,
}

impl<A, B, C, D> SectionPowerRelais<A, B, C, D>
where
    A: OutputPin,
    B: OutputPin,
    C: OutputPin,
    D: OutputPin,
{
    /// When switching between power levels (not from or to 0%), delay for 10ms
    const SWITCHING_DELAY: u32 = 10;

    pub fn new(
        mut power_a: A,
        mut power_b: B,
        mut power_c: C,
        mut power_d: D,
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

pub struct Section<A, B, C, D, T> {
    section_id: u32,

    /// The 4-relais board for powering the section
    power_relais: SectionPowerRelais<A, B, C, D>,
    /// The pin for detecting a train on the section
    train_detection: T,

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

impl<A, B, C, D, T> Section<A, B, C, D, T>
where
    A: OutputPin,
    B: OutputPin,
    C: OutputPin,
    D: OutputPin,
    T: InputPin,
{
    pub fn new(
        section_id: u32,
        power_relais: SectionPowerRelais<A, B, C, D>,
        train_detection: T,
    ) -> Self {
        Section {
            section_id,
            power_relais,
            train_detection,
            is_occupied: false,
        }
    }
}

impl<A, B, C, D, T> SectionDelegate for Section<A, B, C, D, T>
where
    A: OutputPin,
    B: OutputPin,
    C: OutputPin,
    D: OutputPin,
    T: InputPin,
{
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
