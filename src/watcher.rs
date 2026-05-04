use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use anyhow::Result;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc::Sender;

use crate::app::message::Message;

const DEBOUNCE_MS: u64 = 300;

pub struct WatcherGuard {
    _watcher: RecommendedWatcher,
    cancel: Option<tokio::sync::oneshot::Sender<()>>,
}

pub fn start_watcher(tx: Sender<Message>, path: PathBuf) -> Result<WatcherGuard> {
    let (raw_tx, raw_rx) = mpsc::channel::<()>();

    let mut watcher = RecommendedWatcher::new(
        move |res: Result<notify::Event, notify::Error>| {
            if res.is_ok() {
                let _ = raw_tx.send(());
            }
        },
        notify::Config::default(),
    )?;

    watcher.watch(&path, RecursiveMode::Recursive)?;

    let git_index = path.join(".git").join("index");
    if git_index.exists() {
        let _ = watcher.watch(&git_index, RecursiveMode::NonRecursive);
    }

    let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel::<()>();

    let (debounce_tx, mut debounce_rx) = tokio::sync::mpsc::channel::<()>(32);
    tokio::task::spawn_blocking(move || {
        while raw_rx.recv().is_ok() {
            let _ = debounce_tx.blocking_send(());
        }
    });

    tokio::spawn(async move {
        let mut timer: Option<tokio::time::Sleep> = None;
        let mut cancel_rx = cancel_rx;

        loop {
            tokio::select! {
                Some(()) = debounce_rx.recv() => {
                    timer = Some(tokio::time::sleep(Duration::from_millis(DEBOUNCE_MS)));
                }
                _ = async {
                    if let Some(t) = timer.take() {
                        t.await;
                    }
                }, if timer.is_some() => {
                    timer = None;
                    let _ = tx.send(Message::FileChanged).await;
                }
                _ = &mut cancel_rx => break,
            }
        }
    });

    Ok(WatcherGuard {
        _watcher: watcher,
        cancel: Some(cancel_tx),
    })
}

impl Drop for WatcherGuard {
    fn drop(&mut self) {
        let _ = self.cancel.take().and_then(|tx| tx.send(()).ok());
    }
}
