use crossterm::event::{KeyEvent, MouseEvent};

use crate::diff::types::FileDiff;

#[derive(Debug)]
pub enum Message {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    DiffLoaded(Vec<FileDiff>),
    DiffError(String),
    GitCommandDone(Result<String, String>),
    ClipboardCopyDone,
    ConfigLoaded(crate::config::types::Config),
    Tick,
}

#[derive(Debug)]
pub enum Command {
    None,
    LoadDiff(crate::diff::types::DiffMode),
    RunGitCommit { message: String },
    RunGitPush { remote: String, branch: String, set_upstream: bool },
    CopyToClipboard(String),
    OpenFile { path: std::path::PathBuf, line: usize },
    OpenUrl(String),
}
