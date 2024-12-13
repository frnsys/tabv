use layout::Flex;
use rapidfuzz::fuzz::RatioBatchComparator;
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
};
use tui_input::Input;

// (file_idx, sheet_idx)
type SheetAddress = (usize, usize);

#[derive(Default)]
pub struct FinderView {
    pub query: Input,
    list_state: ListState,
    results: Vec<(SheetAddress, String)>,
    selected_result: usize,
}
impl FinderView {
    pub fn update_results(&mut self, opts: &[(SheetAddress, String)]) {
        let query = self.query.value();
        let scorer = RatioBatchComparator::new(query.to_lowercase().chars());

        let mut scored_results = vec![];
        for (addr, opt) in opts {
            let score = scorer.similarity(opt.to_lowercase().chars());
            let score = -(score * 1e6).round() as i64;
            scored_results.push((score, (*addr, opt.to_string())))
        }
        scored_results.sort_by_key(|(score, _)| *score);

        self.results.clear();
        self.results
            .extend(scored_results.into_iter().map(|(_, opt)| opt));
    }

    fn render_results(&mut self, area: Rect, buf: &mut Buffer) {
        let results: Vec<_> = self
            .results
            .iter()
            .map(|(_, name)| ListItem::from(name.clone()))
            .collect();
        let highlight_style = (Color::Green, Color::default());

        self.list_state.select(Some(self.selected_result));

        let list = List::new(results).highlight_style(highlight_style);
        StatefulWidget::render(list, area, buf, &mut self.list_state);
    }

    fn render_footer(&self, area: Rect, buf: &mut Buffer) {
        let info_footer = Paragraph::new("<c-j/k>:select")
            .style(Style::new().fg(Color::DarkGray).bg(Color::Rgb(18, 18, 18)))
            .centered();
        info_footer.render(area, buf);
    }

    pub fn select_next(&mut self) {
        if self.selected_result < self.results.len() - 1 {
            self.selected_result += 1;
        } else {
            self.selected_result = 0;
        }
    }

    pub fn select_previous(&mut self) {
        if self.selected_result > 0 {
            self.selected_result -= 1;
        } else {
            self.selected_result = self.results.len() - 1;
        }
    }

    pub fn get_selected(&self) -> Option<SheetAddress> {
        self.results
            .get(self.selected_result)
            .map(|(addr, _)| *addr)
    }
}
impl Widget for &mut FinderView {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let area = popup_area(area);
        let popup = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("Search");
        let body_area = popup.inner(area);

        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ]);
        let [input_area, results_area, footer_area] = vertical.areas(body_area);

        Clear.render(area, buf);
        popup.render(area, buf);

        let input = Line::raw(self.query.value());
        input.render(input_area, buf);

        let results = Block::new()
            .borders(Borders::TOP)
            .border_style(Color::DarkGray)
            .border_set(symbols::border::PLAIN);
        let list_area = results.inner(results_area);
        results.render(results_area, buf);
        self.render_results(list_area, buf);

        self.render_footer(footer_area, buf);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Max(24),
        Constraint::Fill(1),
    ])
    .flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Length(48)]).flex(Flex::Center);
    let [_, area, _] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}
