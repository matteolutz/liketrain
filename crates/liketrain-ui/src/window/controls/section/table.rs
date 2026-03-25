use gpui_component::table::{Column, TableDelegate};
use itertools::Itertools;
use liketrain_core::{SectionId, TrainId, hardware::event::HardwareSectionPower};

pub struct SectionsTableData {
    pub id: SectionId,

    pub occupant: Option<TrainId>,
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
                Column::new("id", "Id"),
                Column::new("occupant", "Occupant"),
                Column::new("reservation", "Reservation"),
                Column::new("queue", "Queue"),
                Column::new("power", "Power"),
            ],
        }
    }

    pub fn set_section_power(&mut self, section_id: SectionId, power: HardwareSectionPower) {
        let Some(row) = self.data.iter_mut().find(|data| data.id == section_id) else {
            return;
        };

        row.power = power;
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

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
    ) -> impl gpui::IntoElement {
        let row = &self.data[row_ix];
        let col = &self.columns[col_ix];

        match col.key.as_str() {
            "id" => row.id.to_string(),
            "occupant" => row
                .occupant
                .as_ref()
                .map(|id| id.to_string())
                .unwrap_or_else(|| "-".to_string()),
            "reservation" => row
                .reservation
                .as_ref()
                .map(|id| id.to_string())
                .unwrap_or_else(|| "-".to_string()),
            "queue" => row
                .queue
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            "power" => format!("{:?}", row.power),
            _ => "todo".to_string(),
        }
    }
}
