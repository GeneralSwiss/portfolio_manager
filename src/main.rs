use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use portfolio_manager::{Portfolio, Repo};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
};
use std::{error::Error, io, time::Duration};
use tokio::{fs, sync::mpsc, task, time}; // re-export your lib types

/// Simple portfolio TUI
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    cmd: Option<Cmd>,

    #[arg(long, value_name = "FILE", global = true)]
    file: Option<std::path::PathBuf>,

    /// Refresh rate in ms
    #[arg(long, default_value_t = 500)]
    tick: u64,
}

#[derive(clap::Subcommand)]
enum Cmd {

    Load {
        #[arg(value_name = "FILE")]
        file: std::path::PathBuf,
    },
    Tui,
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    // shared in-memory repo
    let repo = Repo::new();

    if let Some(portfolio_file) = &cli.file {
        let bytes: Vec<u8> = fs::read(portfolio_file).await?;
        let portfolio: Portfolio = serde_json::from_slice(&bytes)?;
        repo.set(portfolio);
    }

    match &cli.cmd.unwrap_or(Cmd::Tui) {
        Cmd::Load { file } => {
            return Ok(());
        } 
        Cmd::Tui => run_tui(repo, cli.tick).await?,
    };
    Ok(())
}

async fn run_tui(repo: Repo, tick: u64) -> std::io::Result<()> {
    // channel for tick events
    let (tx, mut rx) = mpsc::channel::<()>(10);
    let tick_ms = tick;
    task::spawn(async move {
        loop {
            time::sleep(Duration::from_millis(tick_ms)).await;
            let _ = tx.send(()).await;
        }
    });

    // ----- terminal init -----
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // main loop
    loop {
        // draw
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(100)].as_ref())
                .split(f.area());

            let p = repo.get(); // snapshot
            let txt = format!("Cash: ${:.2}\nPositions: {}", p.cash, p.positions.len());

            let block = Block::default().title("Portfolio").borders(Borders::ALL);
            f.render_widget(block, chunks[0]);
            f.render_widget(Paragraph::new(txt), chunks[0]);
        })?;

        tokio::select! {
            _ = rx.recv() => { /* tick => redraw next loop */ }
            Ok(ev) = read_event_blocking() => {
                match ev {
                    Event::Key(k) => {
                        if k.code == KeyCode::Char('q') {
                            break;
                        }
                    },
                    _ => { },
                }
            }
        }
    }

    // ----- restore terminal -----
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

async fn read_event_blocking() -> std::io::Result<Event> {
    tokio::task::spawn_blocking(|| crossterm::event::read())
        .await
        .expect("task panicked")
}
