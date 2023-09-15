use itertools::Itertools;
use ratatui::{
    prelude::{Buffer, Rect, *},
    widgets::{Block, Borders, Cell, Row, StatefulWidget, Table, Widget},
};
use std::borrow::Cow;

type Address = u32;

pub trait MemoryProvider {
    /// Reads values starting from `pointer` into the buffer.
    fn read_to_buf(&self, pointer: Address, buf: &mut [Option<u8>]);
}

struct MemoryViewLayout {
    info_bar: Rect,
    address_column: Rect,
    memory_table: Rect,
    ascii_table: Rect,
}

pub struct MemoryViewState {
    /// The memory address being pointed at.
    pub pointer: Address,

    memory_buffer: Vec<Option<u8>>,
    constraints_buffer: Vec<Constraint>,
    beginning_bucket: Address,
    bytes_per_bucket: u16,
}

impl MemoryViewState {
    pub fn new(pointer: Address) -> Self {
        Self {
            pointer,
            memory_buffer: Vec::new(),
            constraints_buffer: Vec::new(),
            beginning_bucket: 0,
            bytes_per_bucket: 0,
        }
    }

    pub fn pointer_index(&self) -> usize {
        self.pointer.abs_diff(self.beginning_bucket) as usize
    }

    pub fn bytes_per_bucket(&self) -> u16 {
        self.bytes_per_bucket
    }
}

pub struct MemoryView<'a> {
    /// The memory provider.
    memory_provider: &'a dyn MemoryProvider,

    /// Block to draw inside.
    block: Option<Block<'a>>,
}

impl<'a> MemoryView<'a> {
    pub fn new(memory_provider: &'a dyn MemoryProvider) -> Self {
        Self {
            memory_provider,
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

    fn main_layout(&mut self, area: Rect) -> MemoryViewLayout {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(4)].as_ref())
            .split(area);

        let view_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length(11),
                    Constraint::Length(1),
                    Constraint::Min(8),
                ]
                .as_ref(),
            )
            .split(main_chunks[0]);

        let info_bar = main_chunks[1];
        let address_column = view_chunks[0];

        let byte_count = view_chunks[2].width / 4;
        let data_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Min(byte_count * 3),
                    Constraint::Length(byte_count + 5),
                ]
                .as_ref(),
            )
            .split(view_chunks[2]);

        let memory_table = data_chunks[0];
        let ascii_table = data_chunks[1];

        MemoryViewLayout {
            info_bar,
            address_column,
            memory_table,
            ascii_table,
        }
    }

    fn render_address_column(&mut self, area: Rect, buf: &mut Buffer, state: &MemoryViewState) {
        let addresses = (0..area.height)
            .map(|index| {
                state
                    .beginning_bucket
                    .checked_add((state.bytes_per_bucket * index) as Address)
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

    fn render_memory_table(&mut self, area: Rect, buf: &mut Buffer, state: &mut MemoryViewState) {
        state
            .constraints_buffer
            .resize(state.bytes_per_bucket as usize, Constraint::Length(2));

        let chunks = state
            .memory_buffer
            .iter()
            .enumerate()
            .chunks(state.bytes_per_bucket as usize);

        let buckets = chunks.into_iter().map(|bytes| {
            let columns_iter = bytes.into_iter().map(|(i, byte)| {
                let cell = Cell::from(
                    byte.map(|x| Cow::from(format!("{x:02X}")))
                        .unwrap_or(Cow::from("◦◦")),
                );

                let color = colorous::COOL.eval_rational(byte.unwrap_or(0) as usize, 256usize);
                let style = {
                    let style = Style::default().fg(Color::Rgb(color.r, color.g, color.b));

                    let style = if ((state.beginning_bucket + i as u32) / 4) % 2 == 0 {
                        style.underlined()
                    } else {
                        style
                    };

                    if i == state.pointer_index() {
                        style.bold().on_light_red()
                    } else {
                        style
                    }
                };
                cell.style(style)
            });

            Row::new(columns_iter)
        });

        let memory_table = Table::new(buckets).widths(&state.constraints_buffer);
        Widget::render(memory_table, area, buf);
    }

    fn render_ascii_table(&mut self, area: Rect, buf: &mut Buffer, state: &MemoryViewState) {
        let constraint = &[Constraint::Percentage(100)];
        let chunks = state
            .memory_buffer
            .iter()
            .chunks(state.bytes_per_bucket as usize);

        let buckets = chunks.into_iter().map(|bytes| {
            let mut result = String::with_capacity(state.bytes_per_bucket as usize);
            for byte in bytes {
                let c = byte.unwrap_or(b' ') as char;
                let c = if !c.is_ascii() {
                    '∘'
                } else if c.is_ascii_control() {
                    '∙'
                } else {
                    c
                };

                result.push(c);
            }

            let mut text = Text::from(result);
            text.lines[0].alignment = Some(Alignment::Center);

            Row::new([text]).style(Style::default().light_blue())
        });

        let block = Block::new().borders(Borders::LEFT);
        let inner_area = block.inner(area);
        block.render(area, buf);

        let ascii_table = Table::new(buckets).widths(constraint.as_slice());
        Widget::render(ascii_table, inner_area, buf);
    }

    pub fn render_info_bar(&mut self, area: Rect, buf: &mut Buffer, state: &MemoryViewState) {
        let block = Block::new().borders(Borders::TOP);
        let inner_area = block.inner(area);
        block.render(area, buf);

        let bytes = &state.memory_buffer[state.pointer_index()..state.pointer_index() + 4];

        let as_u8 = state.memory_buffer[state.pointer_index()].unwrap();
        let as_i8 = as_u8 as i8;

        let as_u16 = match bytes[..2] {
            [Some(a), Some(b)] => Some(u16::from_le_bytes([a, b])),
            _ => None,
        };
        let as_i16 = as_u16.map(|x| x as i16);

        let as_u32 = match bytes[..] {
            [Some(a), Some(b), Some(c), Some(d)] => Some(u32::from_le_bytes([a, b, c, d])),
            _ => None,
        };
        let as_i32 = as_u32.map(|x| x as i32);

        let as_f32 = match bytes[..] {
            [Some(a), Some(b), Some(c), Some(d)] => Some(f32::from_le_bytes([a, b, c, d])),
            _ => None,
        };

        let rows: [[Text; 3]; 3] = [
            [
                format!("u8: {as_u8:?}").into(),
                if let Some(n) = as_u16 {
                    format!("u16: {n:?}").into()
                } else {
                    "u16: --".into()
                },
                if let Some(n) = as_u32 {
                    format!("u32: {n:?}").into()
                } else {
                    "u32: --".into()
                },
            ],
            [
                format!("i8: {as_i8:?}").into(),
                if let Some(n) = as_i16 {
                    format!("i16: {n:?}").into()
                } else {
                    "i16: --".into()
                },
                if let Some(n) = as_i32 {
                    format!("i32: {n:?}").into()
                } else {
                    "i32: --".into()
                },
            ],
            [
                if let Some(n) = as_f32 {
                    format!("f32: {n:?}").into()
                } else {
                    "f32: --".into()
                },
                format!("Selected: {:08X}", state.pointer).into(),
                "Little Endian".into(),
            ],
        ];

        let rows = rows
            .into_iter()
            .map(Row::new)
            .map(|row| row.style(Style::default().light_green()));

        let constraints = [
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(33),
        ];

        let table = Table::new(rows).widths(&constraints);
        Widget::render(table, inner_area, buf);
    }
}

impl<'a> StatefulWidget for MemoryView<'a> {
    type State = MemoryViewState;

    fn render(mut self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let area = self.wrap_in_block(area, buf);
        let layout = self.main_layout(area);

        // update state
        state.bytes_per_bucket = layout.memory_table.width / 3;
        let pointed_bucket = state.pointer - state.pointer % state.bytes_per_bucket as Address;
        state.beginning_bucket = pointed_bucket
            .saturating_sub((state.bytes_per_bucket * layout.address_column.height / 2) as Address);

        let value_count = state.bytes_per_bucket as usize * area.height as usize;
        state.memory_buffer.clear();
        state.memory_buffer.resize(value_count, None);
        self.memory_provider
            .read_to_buf(state.beginning_bucket, &mut state.memory_buffer);

        // render!
        self.render_address_column(layout.address_column, buf, state);
        self.render_memory_table(layout.memory_table, buf, state);
        self.render_ascii_table(layout.ascii_table, buf, state);
        self.render_info_bar(layout.info_bar, buf, state);
    }
}
