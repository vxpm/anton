use anton::{MemoryProvider, MemoryView, MemoryViewState};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
    Terminal,
};
use std::time::Duration;

fn main() -> eyre::Result<()> {
    let mut stdout = std::io::stdout();

    // setup terminal
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;

    run()?;

    // clear terminal
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen,)?;

    Ok(())
}

struct DummyProvider;

impl MemoryProvider for DummyProvider {
    fn read_to_buf(&self, pointer: u32, buf: &mut [Option<u8>]) {
        for (i, value) in buf.iter_mut().enumerate() {
            *value = pointer.checked_add(i as u32).map(|x| x as u8);
        }
    }
}

fn run() -> eyre::Result<()> {
    let stdout = std::io::stdout();
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    let mut state = MemoryViewState::new(0);
    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(80),
                        Constraint::Percentage(10),
                    ]
                    .as_ref(),
                )
                .split(frame.size());

            let block = Block::default().title("Block").borders(Borders::ALL);
            frame.render_widget(block, chunks[0]);

            let block = Block::default()
                .title("Memory Viewer")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL);
            let memory_view = MemoryView::new(&DummyProvider).block(block);

            frame.render_stateful_widget(memory_view, chunks[1], &mut state);

            let block = Block::default().title("Block 3").borders(Borders::ALL);
            frame.render_widget(block, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(500))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('j') => {
                        state.pointer = state
                            .pointer
                            .checked_add(state.bytes_per_bucket() as u32)
                            .unwrap_or(state.pointer)
                    }
                    KeyCode::Char('k') => {
                        state.pointer = state
                            .pointer
                            .checked_sub(state.bytes_per_bucket() as u32)
                            .unwrap_or(state.pointer)
                    }
                    KeyCode::Char('l') => state.pointer = state.pointer.saturating_add(1),
                    KeyCode::Char('h') => state.pointer = state.pointer.saturating_sub(1),
                    KeyCode::Char('q') => break,
                    _ => (),
                }
            }
        }
    }
    Ok(())
}
