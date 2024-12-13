use ratatui::{
    prelude::*,
    widgets::{Block, List, ListItem, ListState, Padding},
};

use crate::TableFile;

use super::TableView;

pub struct FileView {
    file: TableFile,
    table_view: TableView,
    list_state: ListState,
    selected_sheet: usize,
}
impl FileView {
    pub fn new(file: TableFile) -> Self {
        Self {
            file,
            selected_sheet: 0,
            list_state: ListState::default(),
            table_view: TableView::default(),
        }
    }

    pub fn name(&self) -> &str {
        &self.file.name
    }

    pub fn search_options(&self) -> Vec<String> {
        match &self.file.records {
            None => vec![self.name().to_string()],
            Some(records) => {
                let name = self.name();
                records
                    .iter()
                    .map(|(sheet, _)| format!("{}/{}", name, sheet))
                    .collect()
            }
        }
    }

    pub fn select_sheet(&mut self, sheet_idx: usize) {
        self.selected_sheet = sheet_idx;
    }

    pub fn next_sheet(&mut self) {
        if self.selected_sheet < self.file.n_sheets() - 1 {
            self.selected_sheet += 1;
        } else {
            self.selected_sheet = 0;
        }
        self.try_load_file();
    }

    pub fn previous_sheet(&mut self) {
        if self.selected_sheet > 0 {
            self.selected_sheet -= 1;
        } else {
            self.selected_sheet = self.file.n_sheets() - 1;
        }
        self.try_load_file();
    }

    pub fn next_row(&mut self) {
        self.table_view.next_row();
    }

    pub fn previous_row(&mut self) {
        self.table_view.previous_row();
    }

    pub fn next_column(&mut self) {
        self.table_view.next_column();
    }

    pub fn previous_column(&mut self) {
        self.table_view.previous_column();
    }

    pub fn try_load_file(&mut self) {
        if self.file.records.is_none() {
            self.file.load().unwrap();
        }
        let records = self
            .file
            .records
            .as_ref()
            .and_then(|records| records.get(self.selected_sheet).map(|(_, recs)| recs));
        if let Some(records) = records {
            self.table_view.update_shape(records);
        }
    }

    pub fn render_sheet_list(&mut self, area: Rect, buf: &mut Buffer) {
        match &self.file.records {
            None => (),
            Some(records) => {
                if records.len() > 1 {
                    let titles: Vec<_> = records
                        .iter()
                        .map(|(name, _)| ListItem::from(name.to_string()))
                        .collect();
                    let highlight_style = (Color::Green, Color::default());
                    let block = Block::new().padding(Padding::horizontal(1));

                    self.list_state.select(Some(self.selected_sheet));

                    let list = List::new(titles)
                        .block(block)
                        .highlight_style(highlight_style);
                    StatefulWidget::render(list, area, buf, &mut self.list_state);
                }
            }
        }
    }
}
impl Widget for &mut FileView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let records = self
            .file
            .records
            .as_ref()
            .and_then(|records| records.get(self.selected_sheet).map(|(_, recs)| recs));
        match records {
            None => (),
            Some(records) => {
                self.table_view.render(records, area, buf);
            }
        }
    }
}
