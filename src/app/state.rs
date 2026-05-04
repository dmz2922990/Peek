use std::path::PathBuf;

use crate::config::types::Config;
use crate::diff::types::{DiffMode, FileDiff};

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    FileTreeFocus,
    DiffViewFocus,
    Editor,
    GitCommit,
    GitPush,
    FindBar,
    VisualSelect,
}

#[derive(Debug, Clone)]
pub struct FileTreeState {
    pub selected: usize,
    pub scroll: usize,
    pub visible: bool,
}

impl Default for FileTreeState {
    fn default() -> Self {
        Self {
            selected: 0,
            scroll: 0,
            visible: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiffViewState {
    pub scroll: usize,
    pub cursor: usize,
    pub mode: DiffMode,
    pub selected_file: Option<usize>,
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
    pub search_term: Option<String>,
    pub search_matches: Vec<usize>,
    pub current_match: usize,
    pub commit_message: String,
    pub status_message: Option<String>,
    pub extra_context: usize,
    pub total_lines: usize,
    pub viewport_height: usize,
    /// Maps each rendered line index to (hunk_idx, Some(line_idx)) for diff lines,
    /// (hunk_idx, None) for expanded context, or None for headers/indicators
    pub rendered_line_map: Vec<Option<(usize, Option<usize>)>>,
    /// Which hunk to expand context around, set when user presses =
    pub expand_hunk: Option<usize>,
}

impl Default for DiffViewState {
    fn default() -> Self {
        Self {
            scroll: 0,
            cursor: 0,
            mode: DiffMode::Head,
            selected_file: None,
            selection_start: None,
            selection_end: None,
            search_term: None,
            search_matches: Vec::new(),
            current_match: 0,
            commit_message: String::new(),
            status_message: None,
            extra_context: 0,
            total_lines: 0,
            viewport_height: 0,
            rendered_line_map: Vec::new(),
            expand_hunk: None,
        }
    }
}

pub struct App {
    pub mode: AppMode,
    pub config: Config,
    pub diff_data: Vec<FileDiff>,
    pub file_tree: FileTreeState,
    pub diff_view: DiffViewState,
    pub should_quit: bool,
    pub size: (u16, u16),
    pub current_branch: Option<String>,
    pub editing_file: Option<PathBuf>,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            mode: AppMode::DiffViewFocus,
            config,
            diff_data: Vec::new(),
            file_tree: FileTreeState::default(),
            diff_view: DiffViewState::default(),
            should_quit: false,
            size: (80, 24),
            current_branch: None,
            editing_file: None,
        }
    }

    pub fn current_file(&self) -> Option<&FileDiff> {
        let idx = self.file_tree.selected;
        self.diff_data.get(idx)
    }

    pub fn total_additions(&self) -> usize {
        self.diff_data.iter().map(|f| f.additions).sum()
    }

    pub fn total_deletions(&self) -> usize {
        self.diff_data.iter().map(|f| f.deletions).sum()
    }

    pub fn files_changed(&self) -> usize {
        self.diff_data.len()
    }
}
