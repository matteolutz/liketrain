use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::SectionId;

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackSectionWaypoint {}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackSectionGeometry {
    /// The length of this section in meters (already in respect to the tracks scale)
    pub length: f32,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub waypoints: Vec<TrackSectionWaypoint>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TrackGeometry {
    sections: HashMap<SectionId, TrackSectionGeometry>,
}

impl TrackGeometry {
    pub fn section(&self, section_id: &SectionId) -> Option<&TrackSectionGeometry> {
        self.sections.get(section_id)
    }
}
