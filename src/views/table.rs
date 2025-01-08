use std::iter;

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
    n_cols: usize,
    col_offset: usize,
    extra_cols: usize,
    vertical_scroll_state: ScrollbarState,
}
impl TableView {
    pub fn update_shape(&mut self, records: &Records) {
        self.col_widths = constraint_len_calculator(&records.headers, &records.rows);
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

        // Figure out how many columns we can display on scren.
        // WARN: This can be improved as if we have e.g. a screen
        // width of 10 and a column of width >10 then nothing
        // shows up; in that case we should truncate that column to fit.
        let mut fit_cols = 0;
        let mut n_cols = 0;
        let start_idx = self.col_offset;
        for col_width in self.col_widths.iter().skip(start_idx) {
            if fit_cols + col_width > area.width {
                break;
            } else {
                n_cols += 1;
                fit_cols += col_width;
            }
        }
        let total_cols = self.col_widths.len();
        let extra_cols = total_cols - (start_idx + n_cols);
        self.n_cols = n_cols;
        self.extra_cols = extra_cols;

        // Placeholder indicating additional columns to the left.
        // the `take(extra_cols)` bit ensures that if there are no
        // extra columns this will be an empty iterator.
        let extra_col_left =
            iter::once(Cell::from(Text::from("<<").alignment(Alignment::Left))).take(start_idx);

        // Placeholder indicating additional columns to the right.
        // the `take(extra_cols)` bit ensures that if there are no
        // extra columns this will be an empty iterator.
        let extra_col_right =
            iter::once(Cell::from(Text::from(">>").alignment(Alignment::Right))).take(extra_cols);

        let header = extra_col_left
            .clone()
            .chain(
                records
                    .headers
                    .iter()
                    .skip(start_idx)
                    .take(n_cols)
                    .map(Cell::from),
            )
            .chain(extra_col_right.clone())
            .collect::<Row>()
            .style(header_style)
            .height(1);
        let rows = records.rows.iter().enumerate().map(|(i, data)| {
            let color = match i % 2 {
                0 => Color::Reset,
                _ => Color::Rgb(32, 32, 32),
            };
            extra_col_left
                .clone()
                .chain(
                    data.iter()
                        .skip(start_idx)
                        .take(n_cols)
                        .map(|val| Cell::from(Text::from(val).alignment(Alignment::Right))),
                )
                .chain(extra_col_right.clone())
                .collect::<Row>()
                .style(Style::new().fg(Color::Reset).bg(color))
        });
        let bar = " █ ";

        let scroll_indication_left = iter::once(Constraint::Min(1)).take(start_idx);
        let scroll_indication_right = iter::once(Constraint::Min(1)).take(extra_cols);
        let widths = scroll_indication_left
            .chain(
                self.col_widths
                    .iter()
                    .skip(start_idx)
                    .take(n_cols)
                    .map(|width| {
                        // + 1 is for padding.
                        Constraint::Length(width + 1)
                    }),
            )
            .chain(scroll_indication_right);

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
        let info_footer = Paragraph::new("j/k:row h/l:col m:maximize ;:find sheet")
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
        match self.state.selected_column() {
            None => {
                self.state.select_next_column();
            }
            Some(idx) => {
                if idx >= self.n_cols && self.extra_cols > 0 {
                    self.col_offset += 1;
                } else {
                    self.state.select_next_column();
                }
            }
        }
    }

    pub fn previous_column(&mut self) {
        match self.state.selected_column() {
            None => {
                self.state.select_previous_column();
            }
            Some(idx) => {
                if idx <= 1 && self.col_offset > 0 {
                    self.col_offset -= 1;
                } else {
                    self.state.select_previous_column();
                }
            }
        }
    }
}

fn constraint_len_calculator(cols: &StringRecord, items: &[StringRecord]) -> Vec<u16> {
    let mut max_lens: Vec<_> = cols
        .iter()
        .map(|col| UnicodeWidthStr::width(col) as u16)
        .collect();
    for row in items {
        for (i, value) in row.iter().enumerate() {
            max_lens[i] = max_lens[i].max(UnicodeWidthStr::width(value) as u16);
        }
    }
    max_lens
}
