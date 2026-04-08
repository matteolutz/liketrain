use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::SectionId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrackSectionWaypointType {
    Custom {
        name: String,
        should_highlight: bool,
    },

    Station {
        name: String,
    },
}

impl TrackSectionWaypointType {
    pub fn should_highlight(&self) -> bool {
        match self {
            TrackSectionWaypointType::Custom {
                should_highlight, ..
            } => *should_highlight,
            TrackSectionWaypointType::Station { .. } => true,
        }
    }
}

impl std::fmt::Display for TrackSectionWaypointType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrackSectionWaypointType::Custom { name, .. } => write!(f, "{}", name),
            TrackSectionWaypointType::Station { name } => write!(f, "{}", name),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackSectionWaypoint {
    /// The distance along the section in meters (going forward)
    pub at_meter: f32,

    pub r#type: TrackSectionWaypointType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackSectionGeometry {
    /// The length of this section in meters (already in respect to the tracks scale)
    pub length: f32,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub waypoints: Vec<TrackSectionWaypoint>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrackGeometry {
    sections: HashMap<SectionId, TrackSectionGeometry>,
}

impl TrackGeometry {
    pub fn section(&self, section_id: &SectionId) -> Option<&TrackSectionGeometry> {
        self.sections.get(section_id)
    }
}
