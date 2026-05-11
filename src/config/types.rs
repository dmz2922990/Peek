use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub keybindings: Keybindings,
    #[serde(default)]
    pub diff: DiffConfig,
    #[serde(default)]
    pub review: ReviewConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_review_model")]
    pub model: String,
    #[serde(default = "default_review_base_url")]
    pub base_url: String,
    #[serde(default = "default_review_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_review_context_lines")]
    pub context_lines: ReviewContextLines,
    #[serde(default = "default_review_language")]
    pub language: String,
    #[serde(default)]
    pub prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ReviewContextLines {
    Number(usize),
    Full(String),
}

impl Default for ReviewConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            model: default_review_model(),
            base_url: default_review_base_url(),
            max_tokens: default_review_max_tokens(),
            context_lines: default_review_context_lines(),
            language: default_review_language(),
            prompt: None,
        }
    }
}

impl ReviewContextLines {
    pub fn is_full(&self) -> bool {
        matches!(self, ReviewContextLines::Full(s) if s.to_lowercase() == "full")
    }

    pub fn lines(&self) -> Option<usize> {
        match self {
            ReviewContextLines::Number(n) => Some(*n),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybindings {
    #[serde(default = "default_toggle_file_tree")]
    pub toggle_file_tree: String,
    #[serde(default = "default_quit")]
    pub quit: String,
    #[serde(default = "default_scroll_down")]
    pub scroll_down: String,
    #[serde(default = "default_scroll_up")]
    pub scroll_up: String,
    #[serde(default = "default_open_editor")]
    pub open_editor: String,
    #[serde(default = "default_commit")]
    pub commit: String,
    #[serde(default = "default_push")]
    pub push: String,
    #[serde(default = "default_open_pr")]
    pub open_pr: String,
    #[serde(default = "default_find")]
    pub find: String,
    #[serde(default = "default_copy")]
    pub copy: String,
    #[serde(default = "default_visual_select")]
    pub visual_select: String,
    #[serde(default = "default_context_expand")]
    pub context_expand: String,
    #[serde(default = "default_context_collapse")]
    pub context_collapse: String,
    #[serde(default = "default_context_expand_all")]
    pub context_expand_all: String,
    #[serde(default = "default_context_collapse_all")]
    pub context_collapse_all: String,
    #[serde(default = "default_diff_target_switch")]
    pub diff_target_switch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffConfig {
    #[serde(default = "default_context_lines")]
    pub default_context_lines: usize,
    #[serde(default = "default_file_tree_width_percent")]
    pub file_tree_width_percent: u16,
    #[serde(default = "default_file_tree_position")]
    pub file_tree_position: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            keybindings: Keybindings::default(),
            diff: DiffConfig::default(),
            review: ReviewConfig::default(),
        }
    }
}

impl Default for Keybindings {
    fn default() -> Self {
        Self {
            toggle_file_tree: default_toggle_file_tree(),
            quit: default_quit(),
            scroll_down: default_scroll_down(),
            scroll_up: default_scroll_up(),
            open_editor: default_open_editor(),
            commit: default_commit(),
            push: default_push(),
            open_pr: default_open_pr(),
            find: default_find(),
            copy: default_copy(),
            visual_select: default_visual_select(),
            context_expand: default_context_expand(),
            context_collapse: default_context_collapse(),
            context_expand_all: default_context_expand_all(),
            context_collapse_all: default_context_collapse_all(),
            diff_target_switch: default_diff_target_switch(),
        }
    }
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            default_context_lines: default_context_lines(),
            file_tree_width_percent: default_file_tree_width_percent(),
            file_tree_position: default_file_tree_position(),
        }
    }
}

// Default value functions for serde
fn default_toggle_file_tree() -> String { "ctrl+t".into() }
fn default_quit() -> String { "q".into() }
fn default_scroll_down() -> String { "j".into() }
fn default_scroll_up() -> String { "k".into() }
fn default_open_editor() -> String { "e".into() }
fn default_commit() -> String { "c".into() }
fn default_push() -> String { "p".into() }
fn default_open_pr() -> String { "o".into() }
fn default_find() -> String { "/".into() }
fn default_copy() -> String { "y".into() }
fn default_visual_select() -> String { "v".into() }
fn default_context_expand() -> String { "=".into() }
fn default_context_collapse() -> String { "-".into() }
fn default_context_expand_all() -> String { "+".into() }
fn default_context_collapse_all() -> String { "_".into() }
fn default_diff_target_switch() -> String { "S".into() }
fn default_context_lines() -> usize { 3 }
fn default_file_tree_width_percent() -> u16 { 30 }
fn default_file_tree_position() -> String { "left".into() }
fn default_review_model() -> String { "claude-sonnet-4-20250514".into() }
fn default_review_base_url() -> String { "https://api.anthropic.com".into() }
fn default_review_max_tokens() -> u32 { 16384 }
fn default_review_context_lines() -> ReviewContextLines { ReviewContextLines::Number(10) }
fn default_review_language() -> String { "zh".into() }

pub fn config_path() -> PathBuf {
    dirs_home().join(".config").join("review-helper").join("config.toml")
}

pub const KEYBINDING_ENTRIES: &[(&str, &str)] = &[
    ("toggle_file_tree", "Toggle file tree"),
    ("quit", "Quit"),
    ("scroll_down", "Scroll down"),
    ("scroll_up", "Scroll up"),
    ("open_editor", "Open editor"),
    ("commit", "Git commit"),
    ("push", "Git push"),
    ("open_pr", "Open PR"),
    ("find", "Find"),
    ("copy", "Copy"),
    ("visual_select", "Visual select"),
    ("context_expand", "Context expand"),
    ("context_collapse", "Context collapse"),
    ("context_expand_all", "Expand all context"),
    ("context_collapse_all", "Collapse all context"),
    ("diff_target_switch", "Switch diff target"),
];

impl Keybindings {
    pub fn get_binding(&self, index: usize) -> Option<&str> {
        match index {
            0 => Some(&self.toggle_file_tree),
            1 => Some(&self.quit),
            2 => Some(&self.scroll_down),
            3 => Some(&self.scroll_up),
            4 => Some(&self.open_editor),
            5 => Some(&self.commit),
            6 => Some(&self.push),
            7 => Some(&self.open_pr),
            8 => Some(&self.find),
            9 => Some(&self.copy),
            10 => Some(&self.visual_select),
            11 => Some(&self.context_expand),
            12 => Some(&self.context_collapse),
            13 => Some(&self.context_expand_all),
            14 => Some(&self.context_collapse_all),
            15 => Some(&self.diff_target_switch),
            _ => None,
        }
    }

    pub fn set_binding(&mut self, index: usize, value: String) -> bool {
        match index {
            0 => self.toggle_file_tree = value,
            1 => self.quit = value,
            2 => self.scroll_down = value,
            3 => self.scroll_up = value,
            4 => self.open_editor = value,
            5 => self.commit = value,
            6 => self.push = value,
            7 => self.open_pr = value,
            8 => self.find = value,
            9 => self.copy = value,
            10 => self.visual_select = value,
            11 => self.context_expand = value,
            12 => self.context_collapse = value,
            13 => self.context_expand_all = value,
            14 => self.context_collapse_all = value,
            15 => self.diff_target_switch = value,
            _ => return false,
        }
        true
    }
}

pub const CONFIG_ENTRIES: &[(&str, &str)] = &[
    ("default_context_lines", "Context lines"),
    ("file_tree_width_percent", "File tree width %"),
    ("file_tree_position", "File tree position"),
];

pub const REVIEW_CONFIG_ENTRIES: &[(&str, &str)] = &[
    ("api_key", "API Key"),
    ("model", "Model"),
    ("base_url", "Base URL"),
    ("max_tokens", "Max Tokens"),
    ("context_lines", "Context Lines"),
    ("language", "Language"),
    ("prompt", "Custom Prompt"),
];

impl DiffConfig {
    pub fn get_entry(&self, index: usize) -> Option<String> {
        match index {
            0 => Some(self.default_context_lines.to_string()),
            1 => Some(self.file_tree_width_percent.to_string()),
            2 => Some(self.file_tree_position.clone()),
            _ => None,
        }
    }

    pub fn set_entry(&mut self, index: usize, value: String) -> bool {
        match index {
            0 => {
                if let Ok(v) = value.parse::<usize>() {
                    self.default_context_lines = v;
                } else {
                    return false;
                }
            }
            1 => {
                if let Ok(v) = value.parse::<u16>() {
                    self.file_tree_width_percent = v;
                } else {
                    return false;
                }
            }
            2 => {
                let v = value.to_lowercase();
                if v == "left" || v == "right" {
                    self.file_tree_position = v;
                } else {
                    return false;
                }
            }
            _ => return false,
        }
        true
    }
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

impl ReviewConfig {
    pub fn effective_api_key(&self) -> String {
        std::env::var("PEEK_API_KEY")
            .unwrap_or_else(|_| self.api_key.clone())
    }

    pub fn get_entry(&self, index: usize) -> Option<String> {
        match index {
            0 => Some(if self.api_key.is_empty() { String::new() } else { "*".repeat(self.api_key.len()) }),
            1 => Some(self.model.clone()),
            2 => Some(self.base_url.clone()),
            3 => Some(self.max_tokens.to_string()),
            4 => Some(match &self.context_lines {
                ReviewContextLines::Number(n) => n.to_string(),
                ReviewContextLines::Full(s) => s.clone(),
            }),
            5 => Some(self.language.clone()),
            6 => Some(self.prompt.clone().unwrap_or_default()),
            _ => None,
        }
    }

    pub fn get_entry_raw(&self, index: usize) -> Option<String> {
        match index {
            0 => Some(self.api_key.clone()),
            1 => Some(self.model.clone()),
            2 => Some(self.base_url.clone()),
            3 => Some(self.max_tokens.to_string()),
            4 => Some(match &self.context_lines {
                ReviewContextLines::Number(n) => n.to_string(),
                ReviewContextLines::Full(s) => s.clone(),
            }),
            5 => Some(self.language.clone()),
            6 => Some(self.prompt.clone().unwrap_or_default()),
            _ => None,
        }
    }

    pub fn set_entry(&mut self, index: usize, value: String) -> bool {
        match index {
            0 => self.api_key = value,
            1 => self.model = value,
            2 => self.base_url = value,
            3 => {
                if let Ok(n) = value.parse::<u32>() {
                    self.max_tokens = n.max(5000);
                } else {
                    return false;
                }
            }
            4 => {
                if value.to_lowercase() == "full" {
                    self.context_lines = ReviewContextLines::Full("full".into());
                } else if let Ok(n) = value.parse::<usize>() {
                    self.context_lines = ReviewContextLines::Number(n);
                } else {
                    return false;
                }
            }
            5 => self.language = value,
            6 => self.prompt = if value.is_empty() { None } else { Some(value) },
            _ => return false,
        }
        true
    }
}
