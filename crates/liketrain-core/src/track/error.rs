use thiserror::Error;

use crate::{SectionId, SwitchId};

#[derive(Error, Debug)]
pub enum TrackError {
    #[error("The section with id {0} was not found")]
    SectionNotFound(SectionId),

    #[error("The section with id {0} already exists")]
    SectionAlreadyExists(SectionId),

    #[error("The switch with id {0} was not found")]
    SwitchNotFound(SwitchId),

    #[error("The switch with id {0} already exists")]
    SwitchAlreadyExists(SwitchId),
}
