use crate::Address;
use ratatui::{
    prelude::{Buffer, Rect, *},
    widgets::{Block, Borders, Row, StatefulWidget, Table, Widget},
};
use std::borrow::Cow;

pub trait InstructionDisplay {
    fn instruction_display(&self) -> Line;
}

pub trait InstructionProvider<I> {
    /// Reads instructions starting from `pointer` into the buffer.
    fn read_to_buf(&self, pointer: Address, buf: &mut [Option<I>]);
}

struct InstructionViewLayout {
    address_column: Rect,
    instruction_table: Rect,
}

pub struct InstructionViewState<I> {
    /// The memory address being pointed at.
    pub pointer: Address,

    beggining_address: Address,
    instruction_buffer: Vec<Option<I>>,
}

impl<I> InstructionViewState<I> {
    pub fn new(pointer: Address) -> Self {
        Self {
            pointer,
            beggining_address: 0,
            instruction_buffer: Vec::new(),
        }
    }
}

pub struct InstructionView<'a, I> {
    /// The memory provider.
    instruction_provider: &'a dyn InstructionProvider<I>,

    /// Block to draw inside.
    block: Option<Block<'a>>,
}

impl<'a, I> InstructionView<'a, I>
where
    I: InstructionDisplay,
{
    pub fn new(instruction_provider: &'a dyn InstructionProvider<I>) -> Self {
        Self {
            instruction_provider,
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

    fn layout(&mut self, area: Rect) -> InstructionViewLayout {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(11),
                    Constraint::Length(1),
                    Constraint::Min(8),
                ]
                .as_ref(),
            )
            .split(area);

        let address_column = chunks[0];
        let instruction_table = chunks[2];

        InstructionViewLayout {
            address_column,
            instruction_table,
        }
    }

    fn render_address_column(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        state: &InstructionViewState<I>,
    ) {
        let addresses = (0..area.height)
            .map(|index| {
                state
                    .beggining_address
                    .checked_add((std::mem::size_of::<Address>() * index as usize) as Address)
            })
            .map(|addr| {
                let mut text = Text::from(
                    addr.map(|x| (Cow::from(format!("{x:08X}"))))
                        .unwrap_or(Cow::from("--------")),
                );
                text.lines[0].alignment = Some(Alignment::Center);
                Row::new([text]).style(Style::default().light_magenta())
            });

        let block = Block::new().borders(Borders::RIGHT);
        let inner_area = block.inner(area);
        block.render(area, buf);

        let column_table = Table::new(addresses).widths(&[Constraint::Percentage(100)]);
        Widget::render(column_table, inner_area, buf);
    }

    fn render_instruction_table(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut InstructionViewState<I>,
    ) {
        let mut instructions = Vec::new();
        let mut current = state.beggining_address;
        for instruction in &state.instruction_buffer {
            let Some(instruction) = instruction else {
                instructions.push(Row::new(["--"]));
                continue;
            };

            let prefix = Line::from(if current == state.pointer { ">" } else { " " });
            current += std::mem::size_of::<I>() as u32;

            let instr_text = instruction.instruction_display();
            instructions.push(Row::new([prefix, instr_text]));
        }

        let constraint = [Constraint::Length(1), Constraint::Length(area.width)];
        let instruction_table = Table::new(instructions).widths(&constraint);
        Widget::render(instruction_table, area, buf);
    }
}

impl<'a, I> StatefulWidget for InstructionView<'a, I>
where
    I: InstructionDisplay + Clone,
{
    type State = InstructionViewState<I>;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = self.wrap_in_block(area, buf);
        let layout = self.layout(area);

        // update state
        state.beggining_address = state
            .pointer
            .saturating_sub((layout.address_column.height / 2) as Address * 4);

        let value_count = area.height as usize;
        state.instruction_buffer.clear();
        state.instruction_buffer.resize(value_count, None);
        self.instruction_provider
            .read_to_buf(state.beggining_address, &mut state.instruction_buffer);

        // render!
        self.render_address_column(layout.address_column, buf, state);
        self.render_instruction_table(layout.instruction_table, buf, state);
    }
}
