use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

use review_helper::app::{self, message::{Command, Message}, state::App};
use review_helper::config;
use review_helper::diff::parser;
use review_helper::git_cmd::{diff as git_diff, ops as git_ops};
use review_helper::ui;

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = config::load();
    let mut app = App::new(cfg);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Panic hook to restore terminal
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        default_hook(info);
    }));

    // Load initial diff
    let branch = git_diff::current_branch().ok();
    app.current_branch = branch.clone();

    // Channel for async results
    let (tx, mut rx) = mpsc::channel::<Message>(100);

    // Spawn initial diff load
    spawn_diff_load(tx.clone(), app.diff_view.mode.clone());

    // Main loop
    loop {
        // Render
        terminal.draw(|f| ui::draw(f, &mut app))?;

        // Handle async messages (non-blocking)
        while let Ok(msg) = rx.try_recv() {
            let cmd = app::update(&mut app, msg);
            execute_command(cmd, &tx).await;
        }

        // Poll terminal events
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => {
                    let cmd = app::update(&mut app, Message::Key(key));
                    execute_command(cmd, &tx).await;
                }
                Event::Resize(w, h) => {
                    let cmd = app::update(&mut app, Message::Resize(w, h));
                    execute_command(cmd, &tx).await;
                }
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn spawn_diff_load(tx: mpsc::Sender<Message>, mode: review_helper::diff::types::DiffMode) {
    tokio::spawn(async move {
        let result = tokio::task::spawn_blocking(move || {
            git_diff::run_git_diff(&mode)
        }).await;

        match result {
            Ok(Ok(stdout)) => {
                match parser::parse_diff(&stdout) {
                    Ok(files) => {
                        let _ = tx.send(Message::DiffLoaded(files)).await;
                    }
                    Err(e) => {
                        let _ = tx.send(Message::DiffError(e.to_string())).await;
                    }
                }
            }
            Ok(Err(e)) => {
                let _ = tx.send(Message::DiffError(e.to_string())).await;
            }
            Err(e) => {
                let _ = tx.send(Message::DiffError(e.to_string())).await;
            }
        }
    });
}

async fn execute_command(cmd: Command, tx: &mpsc::Sender<Message>) {
    match cmd {
        Command::None => {}
        Command::LoadDiff(mode) => {
            spawn_diff_load(tx.clone(), mode);
        }
        Command::RunGitCommit { message } => {
            let tx = tx.clone();
            tokio::spawn(async move {
                let result = tokio::task::spawn_blocking(move || {
                    git_ops::git_commit(&message)
                }).await;
                let msg = match result {
                    Ok(Ok(output)) => Message::GitCommandDone(Ok(output)),
                    Ok(Err(e)) => Message::GitCommandDone(Err(e.to_string())),
                    Err(e) => Message::GitCommandDone(Err(e.to_string())),
                };
                let _ = tx.send(msg).await;
            });
        }
        Command::RunGitPush { remote, branch, set_upstream } => {
            let tx = tx.clone();
            tokio::spawn(async move {
                let result = tokio::task::spawn_blocking(move || {
                    git_ops::git_push(&remote, &branch, set_upstream)
                }).await;
                let msg = match result {
                    Ok(Ok(output)) => Message::GitCommandDone(Ok(output)),
                    Ok(Err(e)) => Message::GitCommandDone(Err(e.to_string())),
                    Err(e) => Message::GitCommandDone(Err(e.to_string())),
                };
                let _ = tx.send(msg).await;
            });
        }
        Command::CopyToClipboard(content) => {
            let tx = tx.clone();
            tokio::spawn(async move {
                let result = tokio::task::spawn_blocking(move || {
                    let mut clipboard = arboard::Clipboard::new()
                        .map_err(|e| e.to_string())?;
                    clipboard.set_text(&content)
                        .map_err(|e| e.to_string())
                }).await;
                match result {
                    Ok(Ok(())) => { let _ = tx.send(Message::ClipboardCopyDone).await; }
                    Ok(Err(e)) => {
                        let _ = tx.send(Message::DiffError(format!("Clipboard error: {}", e))).await;
                    }
                    Err(e) => {
                        let _ = tx.send(Message::DiffError(format!("Clipboard error: {}", e))).await;
                    }
                }
            });
        }
        Command::OpenUrl(branch) => {
            let tx = tx.clone();
            tokio::spawn(async move {
                let result = tokio::task::spawn_blocking(move || -> Result<(), String> {
                    let url = git_ops::git_remote_url()
                        .map_err(|e| e.to_string())?;
                    let pr_url = git_ops::construct_pr_url(&url, &branch)
                        .ok_or("failed to construct PR URL")?;
                    git_ops::open_url(&pr_url)
                        .map_err(|e| e.to_string())
                }).await;
                match result {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => {
                        let _ = tx.send(Message::DiffError(format!("Open URL error: {}", e))).await;
                    }
                    Err(e) => {
                        let _ = tx.send(Message::DiffError(format!("Open URL error: {}", e))).await;
                    }
                }
            });
        }
        Command::OpenFile { .. } => {
            // Editor handles this internally
        }
    }
}
