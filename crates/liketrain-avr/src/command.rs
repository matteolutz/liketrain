use alloc::vec::Vec;
use liketrain_hardware::{command::HardwareCommand, event::HardwareEvent};

use crate::{
    mode::LiketrainMode,
    track::{SectionDelegate, SectionError},
};

#[derive(Debug)]
pub enum CommandExecutionError {
    SectionError(SectionError),
}

pub struct CommandExecutionContext<'a> {
    pub mode: LiketrainMode,
    pub event_list: &'a mut Vec<HardwareEvent>,
    pub sections: &'a mut [&'a mut dyn SectionDelegate],
}

pub trait CommandExt {
    fn execute(&self, ctx: &mut CommandExecutionContext) -> Result<bool, CommandExecutionError>;
}

impl CommandExt for HardwareCommand {
    fn execute(&self, ctx: &mut CommandExecutionContext) -> Result<bool, CommandExecutionError> {
        match *self {
            Self::Ping { slave_id, seq } if slave_id == ctx.mode.get_slave_id() => {
                ctx.event_list.push(HardwareEvent::Pong { slave_id, seq });
                Ok(true)
            }
            Self::SetSectionPower { section_id, power } => {
                let Some(section) = ctx
                    .sections
                    .iter_mut()
                    .find(|section| section.section_id() == section_id)
                else {
                    return Ok(false);
                };

                section
                    .set_power(power)
                    .map_err(CommandExecutionError::SectionError)?;

                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
