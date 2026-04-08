use std::rc::Rc;

use gpui::{App, IntoElement, ParentElement, Styled, div, prelude::FluentBuilder};
use gpui_component::{
    ActiveTheme,
    button::Button,
    menu::{DropdownMenu, PopupMenuItem},
    table::{Column, ColumnSort, TableDelegate},
};
use itertools::Itertools;
use liketrain_core::{SectionId, TrainId, hardware::event::HardwareSectionPower, ui::UiCommand};

use crate::controller::{ControllerUiWrapper, UiSectionOccupant};

const ALL_SECTION_POWERS: [HardwareSectionPower; 5] = [
    HardwareSectionPower::Off,
    HardwareSectionPower::Quarter,
    HardwareSectionPower::Half,
    HardwareSectionPower::ThreeQuarters,
    HardwareSectionPower::Full,
];

pub struct SectionsTableData {
    pub id: SectionId,

    pub occupant: UiSectionOccupant,
    pub reservation: Option<TrainId>,
    pub queue: Vec<TrainId>,

    pub power: HardwareSectionPower,
}

pub struct SectionsTableDelegate {
    data: Vec<SectionsTableData>,
    columns: Vec<Column>,
}

impl SectionsTableDelegate {
    pub fn new<I, S>(data: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SectionsTableData>,
    {
        Self {
            data: data.into_iter().map_into().collect(),
            columns: vec![
                Column::new("id", "Id").sortable(),
                Column::new("occupant", "Occupant"),
                Column::new("reservation", "Reservation"),
                Column::new("queue", "Queue"),
                Column::new("power", "Power"),
            ],
        }
    }

    fn find_row(&mut self, section_id: SectionId) -> Option<&mut SectionsTableData> {
        self.data.iter_mut().find(|data| data.id == section_id)
    }

    pub fn update_section_power(&mut self, section_id: SectionId, cx: &App) {
        let Some(row) = self.find_row(section_id) else {
            return;
        };

        let Some(power) = ControllerUiWrapper::state(cx)
            .read(cx)
            .section_state(section_id)
            .map(|state| state.power)
        else {
            return;
        };

        row.power = power;
    }

    pub fn update_section_occupant(&mut self, section_id: SectionId, cx: &App) {
        let Some(row) = self.find_row(section_id) else {
            return;
        };

        let Some(occupant) = ControllerUiWrapper::state(cx)
            .read(cx)
            .section_state(section_id)
            .map(|state| state.occupant)
        else {
            return;
        };

        row.occupant = occupant;
    }

    pub fn update_section_reservation(&mut self, section_id: SectionId, cx: &App) {
        let Some(row) = self.find_row(section_id) else {
            return;
        };

        let Some(reservation) = ControllerUiWrapper::state(cx)
            .read(cx)
            .section_state(section_id)
            .map(|state| state.reserved_by)
        else {
            return;
        };

        row.reservation = reservation;
    }

    pub fn update_section_queue(&mut self, section_id: SectionId, cx: &App) {
        let Some(row) = self.find_row(section_id) else {
            return;
        };

        let Some(queue) = ControllerUiWrapper::state(cx)
            .read(cx)
            .section_state(section_id)
            .map(|state| state.queue.iter().copied().collect::<Vec<_>>())
        else {
            return;
        };

        row.queue = queue;
    }
}

impl TableDelegate for SectionsTableDelegate {
    fn columns_count(&self, _cx: &gpui::App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _cx: &gpui::App) -> usize {
        self.data.len()
    }

    fn column(&self, col_ix: usize, _cx: &gpui::App) -> Column {
        self.columns[col_ix].clone()
    }

    fn perform_sort(
        &mut self,
        col_ix: usize,
        sort: gpui_component::table::ColumnSort,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
    ) {
        let col = &self.columns[col_ix];
        match col.key.as_ref() {
            "id" => match sort {
                ColumnSort::Default | ColumnSort::Ascending => {
                    self.data.sort_by(|a, b| a.id.cmp(&b.id))
                }
                ColumnSort::Descending => self.data.sort_by(|a, b| b.id.cmp(&a.id)),
            },
            _ => unreachable!(),
        }
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut gpui::Window,
        cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
    ) -> impl gpui::IntoElement {
        let row = &self.data[row_ix];
        let col = &self.columns[col_ix];

        match col.key.as_str() {
            "id" => format!("S{}", row.id).into_any_element(),
            "occupant" => row
                .occupant
                .train_id()
                .map(|(id, was_freed)| {
                    div()
                        .when(was_freed, |this| this.text_color(cx.theme().danger))
                        .when(!was_freed, |this| this.text_color(cx.theme().success))
                        .child(
                            ControllerUiWrapper::state(cx)
                                .read(cx)
                                .train(id)
                                .map(|train| train.data.name.clone())
                                .unwrap_or_else(|| id.to_string()),
                        )
                })
                .unwrap_or_else(|| div().child("-".to_string()))
                .into_any_element(),
            "reservation" => row
                .reservation
                .as_ref()
                .map(|id| {
                    ControllerUiWrapper::state(cx)
                        .read(cx)
                        .train(*id)
                        .map(|train| train.data.name.clone())
                        .unwrap_or_else(|| id.to_string())
                })
                .unwrap_or_else(|| "-".to_string())
                .into_any_element(),
            "queue" => row
                .queue
                .iter()
                .map(|id| {
                    ControllerUiWrapper::state(cx)
                        .read(cx)
                        .train(*id)
                        .map(|train| train.data.name.clone())
                        .unwrap_or_else(|| id.to_string())
                })
                .collect::<Vec<_>>()
                .join(", ")
                .into_any_element(),
            "power" => Button::new("power")
                .size_full()
                .label(format!("{:?}", row.power))
                .dropdown_menu({
                    let current_power = row.power;
                    let section_id = row.id;
                    move |mut menu, _, _| {
                        for power in ALL_SECTION_POWERS {
                            menu = menu.item(PopupMenuItem::Item {
                                icon: None,
                                label: format!("{:?}", power).into(),
                                disabled: power == current_power,
                                checked: power == current_power,
                                is_link: false,
                                action: None,
                                handler: Some(Rc::new(move |_, _, cx| {
                                    ControllerUiWrapper::exec(
                                        UiCommand::SetSectionPower { section_id, power },
                                        cx,
                                    );
                                })),
                            });
                        }

                        menu
                    }
                })
                .into_any_element(),
            _ => "todo".to_string().into_any_element(),
        }
    }
}
