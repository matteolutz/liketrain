use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
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
    pub debug_messages: &'a mut Vec<String>,
}

impl<'a> CommandExecutionContext<'a> {
    pub fn debug(&mut self, message: impl AsRef<str>) {
        self.debug_messages.push(message.as_ref().to_string());
    }
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
            Self::ResetAll => {
                for section in ctx.sections.iter_mut() {
                    section
                        .reset()
                        .map_err(CommandExecutionError::SectionError)?;
                }

                // TODO: maybe reset switches?

                Ok(false) // return false, so the command will be forwared to the slaves
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
