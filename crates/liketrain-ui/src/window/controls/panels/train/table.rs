use gpui::{App, AppContext, Bounds, IntoElement, Styled, WindowBounds, WindowOptions};
use gpui_component::{
    button::Button,
    table::{Column, ColumnSort, TableDelegate},
};
use itertools::Itertools;
use liketrain_core::{SectionId, TrainId};

use crate::{
    controller::ControllerUiWrapper,
    ebula::{Ebula, EbulaTheme},
};

pub struct TrainsTableData {
    pub id: TrainId,
    pub current_section: Option<SectionId>,
}

pub struct TrainsTableDelegate {
    data: Vec<TrainsTableData>,
    columns: Vec<Column>,
}

impl TrainsTableDelegate {
    pub fn new<I, S>(data: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<TrainsTableData>,
    {
        Self {
            data: data.into_iter().map_into().collect(),
            columns: vec![
                Column::new("id", "Id").sortable(),
                Column::new("section", "Section"),
                Column::new("ebula", "EBuLa"),
            ],
        }
    }

    fn find_row(&mut self, train_id: TrainId) -> Option<&mut TrainsTableData> {
        self.data.iter_mut().find(|data| data.id == train_id)
    }

    pub fn update_current_section(&mut self, train_id: TrainId, cx: &App) {
        let Some(row) = self.find_row(train_id) else {
            return;
        };

        let Some(current_section) = ControllerUiWrapper::state(cx)
            .read(cx)
            .train(train_id)
            .map(|train| train.current_section)
        else {
            return;
        };

        row.current_section = current_section;
    }
}

impl TableDelegate for TrainsTableDelegate {
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
        _cx: &mut gpui::Context<gpui_component::table::TableState<Self>>,
    ) -> impl gpui::IntoElement {
        let row = &self.data[row_ix];
        let col = &self.columns[col_ix];

        match col.key.as_str() {
            "id" => row.id.to_string().into_any_element(),
            "section" => row
                .current_section
                .map(|section_id| format!("S{}", section_id))
                .unwrap_or_else(|| "-".to_string())
                .into_any_element(),
            "ebula" => Button::new("ebula")
                .label("EBuLa")
                .size_full()
                .on_click({
                    let train_id = row.id;

                    move |_, _, cx| {
                        let bounds = Bounds::centered(None, Ebula::get_window_size(600.0), cx);
                        cx.open_window(
                            WindowOptions {
                                window_bounds: Some(WindowBounds::Windowed(bounds)),
                                ..Default::default()
                            },
                            |window, cx| {
                                window.set_window_title("liketrain - EBuLa");
                                cx.new(|cx| Ebula::new(train_id, EbulaTheme::default_light(), cx))
                            },
                        )
                        .unwrap();
                    }
                })
                .into_any_element(),
            _ => "todo".to_string().into_any_element(),
        }
    }
}
