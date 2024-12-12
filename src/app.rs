use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    text::Line,
    widgets::{Tabs, Widget},
    DefaultTerminal,
};

use crate::{file::TableFile, table::TableView};

pub struct App {
    state: AppState,
    files: Vec<TableFile>,
    table_view: TableView,
    selected_file: usize,
}
impl App {
    pub fn new(files: Vec<TableFile>) -> Self {
        if files.is_empty() {
            panic!("No files found.");
        }

        let mut app = App {
            files,
            selected_file: 0,
            table_view: TableView::default(),
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
                match key.code {
                    KeyCode::Char('L') => self.next_tab(),
                    KeyCode::Char('H') => self.previous_tab(),
                    KeyCode::Char('j') | KeyCode::Down => self.table_view.next_row(),
                    KeyCode::Char('k') | KeyCode::Up => self.table_view.previous_row(),
                    KeyCode::Char('l') | KeyCode::Right => self.table_view.next_column(),
                    KeyCode::Char('h') | KeyCode::Left => self.table_view.previous_column(),
                    KeyCode::Char('q') | KeyCode::Esc => self.quit(),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn next_tab(&mut self) {
        if self.selected_file < self.files.len() - 1 {
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
            self.selected_file = self.files.len() - 1;
        }
        self.try_load_file();
    }

    fn try_load_file(&mut self) {
        let file = &mut self.files[self.selected_file];
        if file.records.is_none() {
            file.load().unwrap();
        }
        self.table_view
            .update_shape(file.records.as_ref().expect("Data was loaded"));
    }

    fn quit(&mut self) {
        self.state = AppState::Quitting;
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::{Length, Min};
        let vertical = Layout::vertical([Length(1), Min(0), Length(1)]);
        let [header_area, inner_area, footer_area] = vertical.areas(area);

        let horizontal = Layout::horizontal([Min(0), Length(4)]);
        let [tabs_area, title_area] = horizontal.areas(header_area);

        render_title(title_area, buf);
        self.render_tabs(tabs_area, buf);

        let file = &self.files[self.selected_file];
        match &file.records {
            None => (),
            Some(records) => {
                self.table_view.render(records, inner_area, buf);
            }
        }
        render_footer(footer_area, buf);
    }
}

impl App {
    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let titles = self.files.iter().map(|file| file.name.to_string());
        let highlight_style = (Color::Green, Color::default());
        Tabs::new(titles)
            .highlight_style(highlight_style)
            .select(self.selected_file)
            .padding("", "")
            .divider(" ")
            .render(area, buf);
    }
}

fn render_title(area: Rect, buf: &mut Buffer) {
    "tabv".bold().render(area, buf);
}

fn render_footer(area: Rect, buf: &mut Buffer) {
    Line::raw("◄ ► to change tab | Press q to quit")
        .centered()
        .render(area, buf);
}
