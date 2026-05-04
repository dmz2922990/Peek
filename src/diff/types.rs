use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileDiff {
    pub old_path: PathBuf,
    pub new_path: PathBuf,
    pub status: FileStatus,
    pub hunks: Vec<Hunk>,
    pub additions: usize,
    pub deletions: usize,
    pub is_binary: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Hunk {
    pub old_start: usize,
    pub old_count: usize,
    pub new_start: usize,
    pub new_count: usize,
    pub function_name: Option<String>,
    pub lines: Vec<DiffLine>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiffLine {
    Context {
        content: String,
        old_line: usize,
        new_line: usize,
    },
    Add {
        content: String,
        new_line: usize,
    },
    Delete {
        content: String,
        old_line: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum DiffMode {
    Head,
    MainBranch,
    OtherBranch(String),
}

impl std::fmt::Display for DiffMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiffMode::Head => write!(f, "HEAD"),
            DiffMode::MainBranch => write!(f, "main"),
            DiffMode::OtherBranch(b) => write!(f, "{}", b),
        }
    }
}

impl FileDiff {
    pub fn display_path(&self) -> &std::path::Path {
        if self.new_path.as_os_str().is_empty() {
            &self.old_path
        } else {
            &self.new_path
        }
    }
}

impl Hunk {
    pub fn new_start(&self) -> usize {
        self.new_start
    }

    pub fn new_end(&self) -> usize {
        self.new_start + self.new_count.saturating_sub(1)
    }
}
