use std::rc::Rc;

use gpui::{
    App, AppContext, Bounds, IntoElement, ParentElement, SharedString, Styled, WindowBounds,
    WindowOptions, div,
};
use gpui_component::{
    ActiveTheme,
    button::Button,
    menu::{DropdownMenu, PopupMenuItem},
    table::{Column, ColumnSort, TableDelegate},
};
use itertools::Itertools;
use liketrain_core::{SectionId, TrainId, TrainSpeed, TrainState, ui::UiCommand};

use crate::{
    controller::ControllerUiWrapper,
    ebula::{Ebula, EbulaTheme},
};

const ALL_TRAIN_SPEEDS: [TrainSpeed; 4] = [
    TrainSpeed::Slow,
    TrainSpeed::Medium,
    TrainSpeed::AlmostFast,
    TrainSpeed::Fast,
];

pub struct TrainsTableData {
    pub id: TrainId,

    pub name: SharedString,

    pub current_section: Option<SectionId>,
    pub speed: TrainSpeed,

    pub state: TrainState,
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
                Column::new("name", "Name").sortable(),
                Column::new("section", "Section"),
                Column::new("state", "State"),
                Column::new("speed", "Speed"),
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

    pub fn update_speed(&mut self, train_id: TrainId, cx: &App) {
        let Some(row) = self.find_row(train_id) else {
            return;
        };
        let Some(speed) = ControllerUiWrapper::state(cx)
            .read(cx)
            .train(train_id)
            .map(|train| train.speed)
        else {
            return;
        };
        row.speed = speed;
    }

    pub fn update_state(&mut self, train_id: TrainId, cx: &App) {
        let Some(row) = self.find_row(train_id) else {
            return;
        };
        let Some(state) = ControllerUiWrapper::state(cx)
            .read(cx)
            .train(train_id)
            .map(|train| train.state)
        else {
            return;
        };
        row.state = state;
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
            "name" => match sort {
                ColumnSort::Default | ColumnSort::Ascending => {
                    self.data.sort_by(|a, b| a.name.cmp(&b.name))
                }
                ColumnSort::Descending => self.data.sort_by(|a, b| b.name.cmp(&a.name)),
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
            "id" => row.id.to_string().into_any_element(),
            "name" => row.name.clone().into_any_element(),
            "section" => row
                .current_section
                .map(|section_id| format!("S{}", section_id))
                .unwrap_or_else(|| "-".to_string())
                .into_any_element(),
            "state" => div()
                .child(format!("{:?}", row.state))
                .text_color(match row.state {
                    TrainState::Default => cx.theme().success,
                    TrainState::Waiting => cx.theme().warning,
                })
                .into_any_element(),
            "speed" => Button::new("speed")
                .size_full()
                .label(format!("{:?}", row.speed))
                .dropdown_menu({
                    let current_speed = row.speed;
                    let train_id = row.id;

                    move |mut menu, _, _| {
                        for speed in ALL_TRAIN_SPEEDS {
                            menu = menu.item(PopupMenuItem::Item {
                                icon: None,
                                label: format!("{:?}", speed).into(),
                                disabled: speed == current_speed,
                                checked: speed == current_speed,
                                is_link: false,
                                action: None,
                                handler: Some(Rc::new(move |_, _, cx| {
                                    ControllerUiWrapper::exec(
                                        UiCommand::SetTrainSpeed { train_id, speed },
                                        cx,
                                    );
                                })),
                            });
                        }

                        menu
                    }
                })
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
