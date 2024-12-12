use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout, Rect},
    style::Color,
    symbols,
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Padding, StatefulWidget, Widget},
    DefaultTerminal,
};

use crate::{file::TableFile, table::TableView};

struct FileView {
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

    fn name(&self) -> &str {
        &self.file.name
    }

    fn next_sheet(&mut self) {
        if self.selected_sheet < self.file.n_sheets() - 1 {
            self.selected_sheet += 1;
        } else {
            self.selected_sheet = 0;
        }
        self.try_load_file();
    }

    fn previous_sheet(&mut self) {
        if self.selected_sheet > 0 {
            self.selected_sheet -= 1;
        } else {
            self.selected_sheet = self.file.n_sheets() - 1;
        }
        self.try_load_file();
    }

    fn next_row(&mut self) {
        self.table_view.next_row();
    }

    fn previous_row(&mut self) {
        self.table_view.previous_row();
    }

    fn next_column(&mut self) {
        self.table_view.next_column();
    }

    fn previous_column(&mut self) {
        self.table_view.previous_column();
    }

    fn try_load_file(&mut self) {
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

    fn render_tabs(&mut self, area: Rect, buf: &mut Buffer) {
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

pub struct App {
    state: AppState,
    file_views: Vec<FileView>,
    list_state: ListState,
    selected_file: usize,
}
impl App {
    pub fn new(files: Vec<TableFile>) -> Self {
        if files.is_empty() {
            panic!("No files found.");
        }

        let file_views = files.into_iter().map(|file| FileView::new(file)).collect();
        let mut app = App {
            file_views,
            selected_file: 0,
            list_state: ListState::default(),
            state: AppState::default(),
        };
        app.try_load_file();

        app
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum AppState {
    #[default]
    Running,
    Quitting,
}

impl App {
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.state == AppState::Running {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                let view = &mut self.file_views[self.selected_file];
                match key.code {
                    KeyCode::Char('J') => self.next_tab(),
                    KeyCode::Char('K') => self.previous_tab(),
                    KeyCode::Char('j') | KeyCode::Down => {
                        if key.modifiers == KeyModifiers::CONTROL {
                            view.next_sheet()
                        } else {
                            view.next_row()
                        }
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        if key.modifiers == KeyModifiers::CONTROL {
                            view.previous_sheet()
                        } else {
                            view.previous_row()
                        }
                    }
                    KeyCode::Char('l') | KeyCode::Right => view.next_column(),
                    KeyCode::Char('h') | KeyCode::Left => view.previous_column(),
                    KeyCode::Char('q') | KeyCode::Esc => self.quit(),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn next_tab(&mut self) {
        if self.selected_file < self.file_views.len() - 1 {
            self.selected_file += 1;
        } else {
            self.selected_file = 0;
        }
        self.try_load_file();
    }

    fn previous_tab(&mut self) {
        if self.selected_file > 0 {
            self.selected_file -= 1;
        } else {
            self.selected_file = self.file_views.len() - 1;
        }
        self.try_load_file();
    }

    fn try_load_file(&mut self) {
        let file = &mut self.file_views[self.selected_file];
        file.try_load_file();
    }

    fn quit(&mut self) {
        self.state = AppState::Quitting;
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::{Length, Min, Percentage};
        let layout = Layout::horizontal([Length(32), Min(0)]);
        let [sidebar_area, table_area] = layout.areas(area);

        let sidebar = Layout::vertical([Percentage(50), Percentage(50)]);
        let [files_area, sheets_area] = sidebar.areas(sidebar_area);

        self.render_tabs(files_area, buf);

        let file = &mut self.file_views[self.selected_file];
        file.render_tabs(sheets_area, buf);
        let block = Block::new()
            .border_style(Color::Red)
            .borders(Borders::LEFT)
            .border_set(symbols::border::PLAIN);
        let inner_table_area = block.inner(table_area);
        block.render(table_area, buf);
        file.render(inner_table_area, buf);
        // render_footer(footer_area, buf);
    }
}

impl App {
    fn render_tabs(&mut self, area: Rect, buf: &mut Buffer) {
        let titles: Vec<_> = self
            .file_views
            .iter()
            .map(|file| ListItem::from(file.name()))
            .collect();
        let highlight_style = (Color::Green, Color::default());
        let block = Block::new()
            .padding(Padding::horizontal(1))
            .border_style(Color::Red)
            .borders(Borders::BOTTOM)
            .border_set(symbols::border::PLAIN);

        self.list_state.select(Some(self.selected_file));

        let list = List::new(titles)
            .block(block)
            .highlight_style(highlight_style);
        StatefulWidget::render(list, area, buf, &mut self.list_state);
    }
}

fn render_footer(area: Rect, buf: &mut Buffer) {
    Line::raw("◄ ► to change tab | Press q to quit")
        .centered()
        .render(area, buf);
}
