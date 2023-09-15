use ratatui::{
    prelude::*,
    widgets::{Block, StatefulWidget, Widget},
};
use std::marker::PhantomData;

pub trait Address {
    /// The length of this address when displayed.
    const DISPLAY_LENGTH: u8;
}

pub trait MemoryProvider<A>
where
    A: Address,
{
    fn write_view(&self, start: A, buf: &mut [Option<u8>]);
}

struct MemoryViewState<A> {
    /// The memory address being currently pointed at.
    pub pointer: A,
}

struct MemoryView<'a, A> {
    /// The memory provider.
    memory_provider: &'a dyn MemoryProvider<A>,

    /// Block to draw inside.
    block: Option<Block<'a>>,

    /// How many bytes there should be in a column.
    group_by: u8,

    _phantom: PhantomData<A>,
}

impl<'provider, A> StatefulWidget for MemoryView<'provider, A>
where
    A: Address,
{
    type State = MemoryViewState<A>;

    fn render(
        mut self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        // 01. draw block and find actual area
        let area = if let Some(block) = self.block.take() {
            let inner_area = block.inner(area);
            block.render(area, buf);
            inner_area
        } else {
            area
        };

        // 02. split area into address column and memory content
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(A::DISPLAY_LENGTH as u16 + 2),
                    Constraint::Min(A::DISPLAY_LENGTH as u16),
                ]
                .as_ref(),
            )
            .split(area);

        let column_area = chunks[0];
        let memory_area = chunks[1];

        // 03. figure out how many columns fit in the memory area

        // 04. render column

        // 05. render memory
    }
}
