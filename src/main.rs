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

fn run() -> eyre::Result<()> {
    let stdout = std::io::stdout();
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    Ok(loop {
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

            let block = Block::default().title("Block 2").borders(Borders::ALL);
            frame.render_widget(block, chunks[1]);

            let block = Block::default().title("Block 3").borders(Borders::ALL);
            frame.render_widget(block, chunks[2]);
        })?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if KeyCode::Char('q') == key.code {
                    break;
                }
            }
        }
    })
}
