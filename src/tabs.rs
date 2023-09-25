use ratatui::{
    prelude::{Buffer, Rect, *},
    widgets::{Block, Paragraph, Widget},
};
use std::borrow::Cow;

pub struct Tabs<'a> {
    pub titles: &'a [Cow<'a, str>],
    pub selected: usize,
    pub block: Option<Block<'a>>,
}

impl<'a> Tabs<'a> {
    pub fn new(titles: &'a [Cow<'a, str>], selected: usize) -> Self {
        Self {
            titles,
            selected,
            block: None,
        }
    }

    pub fn block(self, block: Block<'a>) -> Self {
        Self {
            block: Some(block),
            ..self
        }
    }

    fn wrap_in_block(&mut self, area: Rect, buf: &mut Buffer) -> Rect {
        if let Some(block) = self.block.take() {
            let inner_area = block.inner(area);
            block.render(area, buf);
            inner_area
        } else {
            area
        }
    }
}

impl<'a> Widget for Tabs<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let area = self.wrap_in_block(area, buf);

        let max = self.titles.len() as u32;
        let constraints = vec![Constraint::Ratio(1, max); self.titles.len()];

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(&*constraints)
            .split(area);

        for (index, (title, area)) in self.titles.iter().zip(chunks.iter()).enumerate() {
            let style = if index == self.selected {
                Style::default().bold().underlined().white()
            } else {
                Style::default().dark_gray()
            };
            let paragraph = Paragraph::new(title.clone())
                .alignment(Alignment::Center)
                .style(style);
            paragraph.render(*area, buf);
        }
    }
}
