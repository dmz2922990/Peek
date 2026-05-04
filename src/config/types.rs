use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub keybindings: Keybindings,
    #[serde(default)]
    pub diff: DiffConfig,
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
    #[serde(default = "default_diff_target_switch")]
    pub diff_target_switch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffConfig {
    #[serde(default = "default_context_lines")]
    pub default_context_lines: usize,
    #[serde(default = "default_file_tree_width_percent")]
    pub file_tree_width_percent: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            keybindings: Keybindings::default(),
            diff: DiffConfig::default(),
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
            diff_target_switch: default_diff_target_switch(),
        }
    }
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            default_context_lines: default_context_lines(),
            file_tree_width_percent: default_file_tree_width_percent(),
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
fn default_context_expand() -> String { "+".into() }
fn default_context_collapse() -> String { "-".into() }
fn default_diff_target_switch() -> String { "tab".into() }
fn default_context_lines() -> usize { 3 }
fn default_file_tree_width_percent() -> u16 { 30 }

pub fn config_path() -> PathBuf {
    dirs_home().join(".config").join("review-helper").join("config.toml")
}

fn dirs_home() -> PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}
