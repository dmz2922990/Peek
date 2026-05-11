use crossterm::event::{KeyEvent, MouseEvent};
use std::path::PathBuf;

use crate::app::state::ReviewComment;
use crate::config::types::ReviewConfig;
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
    FileChanged,
    Tick,
    ReviewResult { path: PathBuf, comments: Vec<ReviewComment> },
    ReviewError(String),
}

#[derive(Debug)]
pub enum Command {
    None,
    LoadDiff(crate::diff::types::DiffMode),
    RunGitCommit { message: String },
    RunGitPush { remote: String, branch: String, set_upstream: bool },
    CopyToClipboard(String),
    OpenFile { path: PathBuf, line: usize },
    OpenUrl(String),
    StartReview { file: FileDiff, repo_root: Option<PathBuf>, config: ReviewConfig },
}
