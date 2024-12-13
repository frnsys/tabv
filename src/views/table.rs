use csv::StringRecord;
use ratatui::{prelude::*, widgets::*};
use unicode_width::UnicodeWidthStr;

use crate::file::Records;

const ITEM_HEIGHT: usize = 1;

#[derive(Default)]
pub struct TableView {
    col_widths: Vec<u16>,
    n_rows: usize,

    state: TableState,
    vertical_scroll_state: ScrollbarState,
    horizontal_scroll_state: ScrollbarState,
}
impl TableView {
    pub fn update_shape(&mut self, records: &Records) {
        self.col_widths = constraint_len_calculator(records.headers.len(), &records.rows);
        self.n_rows = records.rows.len();
    }

    fn render_table(&mut self, records: &Records, area: Rect, buf: &mut Buffer) {
        let header_style = Style::default().fg(Color::White).bg(Color::Reset);
        let selected_row_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(Color::Green);
        let selected_col_style = Style::default().fg(Color::Red);
        let selected_cell_style = Style::default()
            .add_modifier(Modifier::REVERSED)
            .fg(Color::Red);

        let header = records
            .headers
            .iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(header_style)
            .height(1);
        let rows = records.rows.iter().enumerate().map(|(i, data)| {
            let color = match i % 2 {
                0 => Color::Reset,
                _ => Color::Rgb(32, 32, 32),
            };
            data.iter()
                .map(|val| Cell::from(Text::from(val).alignment(Alignment::Right)))
                .collect::<Row>()
                .style(Style::new().fg(Color::Reset).bg(color))
        });
        let bar = " █ ";
        let widths = self.col_widths.iter().map(|width| {
            // + 1 is for padding.
            Constraint::Max(width + 1)
        });
        let t = Table::new(rows, widths)
            .header(header)
            .row_highlight_style(selected_row_style)
            .column_highlight_style(selected_col_style)
            .cell_highlight_style(selected_cell_style)
            .highlight_symbol(Text::from(vec![
                "".into(),
                bar.into(),
                bar.into(),
                "".into(),
            ]))
            .bg(Color::Reset)
            .highlight_spacing(HighlightSpacing::Always);
        StatefulWidget::render(t, area, buf, &mut self.state);
    }

    fn render_scrollbar(&mut self, area: Rect, buf: &mut Buffer) {
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .render(
                area.inner(Margin {
                    vertical: 1,
                    horizontal: 1,
                }),
                buf,
                &mut self.vertical_scroll_state,
            );
    }

    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let info_footer = Paragraph::new("j/k:row h/l:col ;:find sheet")
            .style(Style::new().fg(Color::DarkGray).bg(Color::Rgb(18, 18, 18)))
            .centered();
        info_footer.render(area, buf);
    }

    pub fn render(&mut self, records: &Records, area: Rect, buf: &mut Buffer) {
        let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(1)]);
        let rects = vertical.split(area);

        self.render_table(records, rects[0], buf);
        self.render_scrollbar(rects[0], buf);
        self.render_footer(rects[1], buf);
    }

    pub fn next_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.n_rows - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.vertical_scroll_state = self.vertical_scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn previous_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.n_rows - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.vertical_scroll_state = self.vertical_scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn next_column(&mut self) {
        self.state.select_next_column();
    }

    pub fn previous_column(&mut self) {
        self.state.select_previous_column();
    }
}

fn constraint_len_calculator(cols: usize, items: &[StringRecord]) -> Vec<u16> {
    let mut max_lens = vec![0; cols];
    for row in items {
        for (i, value) in row.iter().enumerate() {
            max_lens[i] = max_lens[i].max(UnicodeWidthStr::width(value) as u16);
        }
    }
    max_lens
}
