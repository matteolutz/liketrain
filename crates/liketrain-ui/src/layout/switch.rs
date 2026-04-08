use std::collections::{HashMap, VecDeque};

use gpui::{Pixels, Point, point, px};
use liketrain_core::{SectionEnd, SwitchConnection, SwitchId, SwitchState, Track};
use strum::IntoEnumIterator;

use crate::layout::{Layout, ResolvedLayout};

#[derive(Debug, Copy, Clone, Default)]
pub enum SwitchResolutionState<T> {
    #[default]
    Unresolved,

    Resolved(T),
    Unresolvable,
}

impl<T> SwitchResolutionState<T> {
    pub fn is_unresolved(&self) -> bool {
        matches!(self, SwitchResolutionState::Unresolved)
    }

    pub fn is_resolved(&self) -> bool {
        matches!(self, SwitchResolutionState::Resolved(_))
    }

    pub fn is_unresolvable(&self) -> bool {
        matches!(self, SwitchResolutionState::Unresolvable)
    }

    pub fn ok(&self) -> Option<&T> {
        match self {
            SwitchResolutionState::Resolved(t) => Some(t),
            _ => None,
        }
    }

    pub fn map_resolved<U>(
        self,
        f: impl FnOnce(T) -> SwitchResolutionState<U>,
    ) -> SwitchResolutionState<U> {
        match self {
            SwitchResolutionState::Unresolved => SwitchResolutionState::Unresolved,
            SwitchResolutionState::Resolved(t) => f(t),
            SwitchResolutionState::Unresolvable => SwitchResolutionState::Unresolvable,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, strum::EnumIter)]
pub enum SwitchResolutionEnd {
    From,
    ToLeft,
    ToRight,
}

/// Represents the state of a switch during resolution
#[derive(Debug, Default, Clone)]
pub struct SwitchResolution {
    from: SwitchResolutionState<Point<Pixels>>,

    to_left: SwitchResolutionState<Point<Pixels>>,
    to_right: SwitchResolutionState<Point<Pixels>>,

    /// The direction from the from point to the center point.
    /// We use this, to draw the switch as a continuation from the section it's coming from
    from_to_center_direction: SwitchResolutionState<Point<Pixels>>,
}

impl SwitchResolution {
    pub fn get(&self, end: SwitchResolutionEnd) -> &SwitchResolutionState<Point<Pixels>> {
        match end {
            SwitchResolutionEnd::From => &self.from,
            SwitchResolutionEnd::ToLeft => &self.to_left,
            SwitchResolutionEnd::ToRight => &self.to_right,
        }
    }

    pub fn get_mut(
        &mut self,
        end: SwitchResolutionEnd,
    ) -> &mut SwitchResolutionState<Point<Pixels>> {
        match end {
            SwitchResolutionEnd::From => &mut self.from,
            SwitchResolutionEnd::ToLeft => &mut self.to_left,
            SwitchResolutionEnd::ToRight => &mut self.to_right,
        }
    }

    /// Get the center point of this switch resolution.
    /// This only works if, the `from` and one of the `to` ends are resolved.
    pub fn center_point(&self) -> Option<Point<Pixels>> {
        let from = self.from.ok().copied()?;

        if let Some((to_left, to_right)) =
            self.to_left.ok().copied().zip(self.to_right.ok().copied())
        {
            let to_center = (to_left + to_right) / 2.0;
            Some((from + to_center) / 2.0)
        } else {
            let to = self.to_left.ok().or(self.to_right.ok()).copied()?;
            Some((from + to) / 2.0)
        }
    }
}

impl SwitchResolution {
    pub fn mark_all_unresolvable(&mut self) {
        self.from = SwitchResolutionState::Unresolvable;
        self.to_left = SwitchResolutionState::Unresolvable;
        self.to_right = SwitchResolutionState::Unresolvable;
    }
}

impl Layout {
    pub fn resolve(self, track: &Track) -> ResolvedLayout {
        let mut switch_resolutions: HashMap<SwitchId, SwitchResolution> = HashMap::new();
        let mut switches_to_resolve = track
            .switches()
            .map(|(id, _)| id.clone())
            .collect::<VecDeque<_>>();

        while let Some(switch_id) = switches_to_resolve.pop_front() {
            let resolution = switch_resolutions.entry(switch_id.clone()).or_default();

            let Some(switch) = track.switch(&switch_id) else {
                // switch not found, not resolvable and skip
                resolution.mark_all_unresolvable();
                continue;
            };

            // make the borrow checker happy
            let resolution = resolution.clone();

            for resolution_end in SwitchResolutionEnd::iter() {
                if resolution.get(resolution_end).is_unresolvable() {
                    // skip resolved or unresolvable
                    continue;
                }

                let connection = match resolution_end {
                    SwitchResolutionEnd::From => switch.from(),
                    SwitchResolutionEnd::ToLeft => switch.to(SwitchState::Left),
                    SwitchResolutionEnd::ToRight => switch.to(SwitchState::Right),
                };

                match connection {
                    &SwitchConnection::Section {
                        section_id,
                        section_end,
                    } => {
                        let resolution = switch_resolutions
                            .get_mut(&switch_id)
                            .unwrap()
                            .get_mut(resolution_end);

                        if let Some(layout_section) = self.sections.get(&section_id) {
                            let section_end_point = match section_end {
                                SectionEnd::Start => layout_section.from,
                                SectionEnd::End => layout_section.to(),
                            };

                            *resolution = SwitchResolutionState::Resolved(section_end_point);
                        } else {
                            *resolution = SwitchResolutionState::Unresolvable;
                        }
                    }
                    SwitchConnection::SwitchBack {
                        switch_id: other_switch_id,
                        state,
                    } => {
                        if let Some(other_resolution) =
                            switch_resolutions.get(other_switch_id).cloned()
                        {
                            // we can't really resolve switch to switch connections
                            // but when we have two resolved ends, we can infer the last point
                            let other_connection_end = match state {
                                SwitchState::Left => SwitchResolutionEnd::ToLeft,
                                SwitchState::Right => SwitchResolutionEnd::ToRight,
                            };

                            let mut new_state: SwitchResolutionState<
                                [(SwitchResolutionEnd, Point<Pixels>); 2],
                            > = SwitchResolutionState::Unresolved;

                            let mut first_point: Option<(SwitchResolutionEnd, Point<Pixels>)> =
                                None;

                            for other_end in SwitchResolutionEnd::iter()
                                .filter(|end| *end != other_connection_end)
                            {
                                let other_end_state = other_resolution.get(other_end);

                                match other_end_state {
                                    SwitchResolutionState::Unresolvable => {
                                        new_state = SwitchResolutionState::Unresolvable;
                                        break;
                                    }
                                    SwitchResolutionState::Unresolved => {
                                        // resolve this later
                                        switches_to_resolve.push_back(switch_id.clone());
                                        new_state = SwitchResolutionState::Unresolved;
                                        break;
                                    }
                                    SwitchResolutionState::Resolved(point) => {
                                        if let Some(first_point) = first_point {
                                            new_state = SwitchResolutionState::Resolved([
                                                first_point,
                                                (other_end, *point),
                                            ]);
                                            break;
                                        } else {
                                            first_point = Some((other_end, *point));
                                        }
                                    }
                                }
                            }

                            let new_resolution = new_state.map_resolved(
                                |[(end_a, point_a), (end_b, point_b)]| match (end_a, end_b) {
                                    (SwitchResolutionEnd::ToLeft, SwitchResolutionEnd::ToRight)
                                    | (SwitchResolutionEnd::ToRight, SwitchResolutionEnd::ToLeft) =>
                                    {
                                        // well, we can't resolve this
                                        // this means, we are missing
                                        SwitchResolutionState::Unresolvable
                                    }

                                    (SwitchResolutionEnd::From, SwitchResolutionEnd::ToRight) => {
                                        let x_diff = point_b.x - point_a.x;
                                        let point_res = point_b - point(x_diff, px(0.0));
                                        SwitchResolutionState::Resolved(point_res)
                                    }

                                    (SwitchResolutionEnd::ToRight, SwitchResolutionEnd::From) => {
                                        let x_diff = point_a.x - point_b.x;
                                        let point_res = point_a + point(x_diff, px(0.0));
                                        SwitchResolutionState::Resolved(point_res)
                                    }

                                    (SwitchResolutionEnd::From, SwitchResolutionEnd::ToLeft) => {
                                        let x_diff = point_a.x - point_b.x;
                                        let point_res = point_b + point(x_diff, px(0.0));
                                        SwitchResolutionState::Resolved(point_res)
                                    }

                                    (SwitchResolutionEnd::ToLeft, SwitchResolutionEnd::From) => {
                                        let x_diff = point_b.x - point_a.x;
                                        let point_res = point_a + point(x_diff, px(0.0));
                                        SwitchResolutionState::Resolved(point_res)
                                    }

                                    _ => unreachable!(),
                                },
                            );

                            // assign the position to both switches
                            *switch_resolutions
                                .get_mut(&switch_id)
                                .unwrap()
                                .get_mut(resolution_end) = new_resolution;

                            *switch_resolutions
                                .get_mut(other_switch_id)
                                .unwrap()
                                .get_mut(other_connection_end) = new_resolution;
                        } else {
                            // we don't have a resolution, so the other switch wasn't visited yet
                            // just push ourselves onto the queue again to resolve later
                            switches_to_resolve.push_back(switch_id.clone());
                        }
                    }
                }
            }
        }

        ResolvedLayout {
            layout: self,
            switch_resolutions,
        }
    }
}
