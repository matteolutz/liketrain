use std::rc::Rc;

use gpui::{App, IntoElement, Styled};
use gpui_component::{
    button::Button,
    menu::{DropdownMenu, PopupMenuItem},
    table::{Column, ColumnSort, TableDelegate},
};
use itertools::Itertools;
use liketrain_core::{SwitchId, SwitchState, ui::UiCommand};

use crate::controller::ControllerUiWrapper;

const ALL_SWITCH_STATES: [SwitchState; 2] = [SwitchState::Left, SwitchState::Right];

pub struct SwitchesTableData {
    pub id: SwitchId,

    pub state: SwitchState,
}

pub struct SwitchesTableDelegate {
    data: Vec<SwitchesTableData>,
    columns: Vec<Column>,
}

impl SwitchesTableDelegate {
    pub fn new<I, S>(data: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<SwitchesTableData>,
    {
        Self {
            data: data.into_iter().map_into().collect(),
            columns: vec![
                Column::new("id", "Id").sortable(),
                Column::new("state", "State"),
            ],
        }
    }

    fn find_row(&mut self, switch_id: &SwitchId) -> Option<&mut SwitchesTableData> {
        self.data.iter_mut().find(|data| &data.id == switch_id)
    }

    pub fn update_switch_state(&mut self, switch_id: &SwitchId, cx: &App) {
        let Some(row) = self.find_row(switch_id) else {
            return;
        };

        let Some(state) = ControllerUiWrapper::state(cx)
            .read(cx)
            .switch_state(switch_id)
        else {
            return;
        };

        row.state = *state;
    }
}

impl TableDelegate for SwitchesTableDelegate {
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
            "state" => Button::new("state")
                .size_full()
                .label(format!("{:?}", row.state))
                .dropdown_menu({
                    let current_state = row.state;
                    let switch_id = row.id.clone();

                    move |mut menu, _, _| {
                        for state in ALL_SWITCH_STATES {
                            menu = menu.item(PopupMenuItem::Item {
                                icon: None,
                                label: format!("{:?}", state).into(),
                                disabled: state == current_state,
                                checked: state == current_state,
                                is_link: false,
                                action: None,
                                handler: Some(Rc::new({
                                    let switch_id = switch_id.clone();
                                    move |_, _, cx| {
                                        ControllerUiWrapper::exec(
                                            UiCommand::SetSwitchState {
                                                switch_id: switch_id.clone(),
                                                state,
                                            },
                                            cx,
                                        );
                                    }
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
