#![allow(unused, dead_code)]

use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use portfolio_manager::Error;
use portfolio_manager::{Portfolio, Repo, main_page};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::fs::File;
use std::sync::Mutex;
use std::{io, time::Duration};
use tokio::{fs, sync::mpsc, task, time};
use tracing::info;

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
    Tui,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log_file = "portfolio-manager.log";
    if let Err(e) = setup_logging(log_file) {
        eprintln!("Error initializing logging: {}", e);
        return Err(Error::LoggingInitFailure.into());
    }
    let cli = Cli::parse();

    // shared in-memory repo
    let repo = Repo::new();

    if let Some(portfolio_file) = &cli.file {
        let bytes: Vec<u8> = fs::read(portfolio_file).await?;
        let portfolio: Portfolio = serde_json::from_slice(&bytes)?;
        repo.set(portfolio);
    }

    match &cli.cmd.unwrap_or(Cmd::Tui) {
        Cmd::Tui => run_tui(repo, cli.tick).await?,
    };
    Ok(())
}

async fn run_tui(repo: Repo, tick: u64) -> portfolio_manager::Result<()> {
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
        terminal.draw(|frame| {
            let portfolio = repo.get(); // snapshot
            main_page(frame, &portfolio)
        })?;

        tokio::select! {
            _ = rx.recv() => { /* tick => redraws next loop */ }
            Ok(ev) = read_event_blocking() => {
                info!("Event: {:?}", ev);
                if let Event::Key(k) = ev {
                    if k.code == KeyCode::Char('q') {
                        break;
                    }
                }
            }
        }
    }

    // ----- restore terminal -----
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;
    Ok(())
}

async fn read_event_blocking() -> std::io::Result<Event> {
    tokio::task::spawn_blocking(crossterm::event::read)
        .await
        .expect("task panicked")
}

/// Initialize logging with file-based logging and optional external layers.
///
/// The File should rotate every hour, and the file name should be an indicator of the time in which
/// the logs were created.
fn setup_logging(log_file: &str) -> anyhow::Result<()> {
    let log_file = File::create(log_file)?;
    tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(Mutex::new(log_file))
        .with_level(true)
        .init();
    Ok(())
}
