use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    symbols,
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Padding, StatefulWidget, Widget},
    DefaultTerminal,
};
use tui_input::backend::crossterm::EventHandler;

use crate::{file::TableFile, views::*};

pub struct App {
    state: AppState,
    finder: FinderView,
    finding: bool,
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
            finding: false,
            list_state: ListState::default(),
            finder: FinderView::default(),
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
                if self.finding {
                    match key.code {
                        KeyCode::Enter => {
                            if let Some((file_id, sheet_id)) = self.finder.get_selected() {
                                self.selected_file = file_id;
                                self.file_views[file_id].select_sheet(sheet_id);
                                self.try_load_file();
                                self.finding = false;
                            }
                        }
                        KeyCode::Esc => {
                            self.finding = false;
                        }
                        _ => {
                            if key.modifiers == KeyModifiers::CONTROL {
                                match key.code {
                                    KeyCode::Char('j') | KeyCode::Down => self.finder.select_next(),
                                    KeyCode::Char('k') | KeyCode::Up => {
                                        self.finder.select_previous()
                                    }
                                    _ => (),
                                }
                            } else {
                                let opts: Vec<_> = self
                                    .file_views
                                    .iter()
                                    .enumerate()
                                    .map(|(i, file)| {
                                        file.search_options()
                                            .into_iter()
                                            .enumerate()
                                            .map(move |(j, name)| ((i, j), name))
                                    })
                                    .flatten()
                                    .collect();
                                self.finder.query.handle_event(&Event::Key(key));
                                self.finder.update_results(&opts);
                            }
                        }
                    }
                } else {
                    let view = &mut self.file_views[self.selected_file];
                    match key.code {
                        KeyCode::Char('J') => self.next_file(),
                        KeyCode::Char('K') => self.previous_file(),
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
                        KeyCode::Char(';') => self.finding = true,
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn next_file(&mut self) {
        if self.selected_file < self.file_views.len() - 1 {
            self.selected_file += 1;
        } else {
            self.selected_file = 0;
        }
        self.try_load_file();
    }

    fn previous_file(&mut self) {
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

        let sidebar = Layout::vertical([Percentage(50), Percentage(50), Length(1)]);
        let [files_area, sheets_area, side_footer] = sidebar.areas(sidebar_area);

        self.render_tabs(files_area, buf);

        let file = &mut self.file_views[self.selected_file];
        file.render_sheet_list(sheets_area, buf);

        render_footer(side_footer, buf);

        let block = Block::new()
            .border_style(Color::Red)
            .borders(Borders::LEFT)
            .border_set(symbols::border::PLAIN);
        let inner_table_area = block.inner(table_area);
        block.render(table_area, buf);
        file.render(inner_table_area, buf);

        if self.finding {
            self.finder.render(area, buf);
        }
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
    Line::raw("J/K:file <c-j/k>:sheet")
        .centered()
        .style(Style::new().fg(Color::DarkGray).bg(Color::Rgb(18, 18, 18)))
        .render(area, buf);
}
