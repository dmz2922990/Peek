use review_helper::config::{load_from_str, types::Config};

#[test]
fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.keybindings.quit, "q");
    assert_eq!(config.keybindings.toggle_file_tree, "ctrl+t");
    assert_eq!(config.keybindings.copy, "y");
    assert_eq!(config.diff.default_context_lines, 3);
    assert_eq!(config.diff.file_tree_width_percent, 30);
}

#[test]
fn test_load_valid_toml() {
    let toml = r#"
[keybindings]
quit = "Q"
copy = "Y"

[diff]
default_context_lines = 5
"#;
    let config = load_from_str(toml).unwrap();
    assert_eq!(config.keybindings.quit, "Q");
    assert_eq!(config.keybindings.copy, "Y");
    assert_eq!(config.diff.default_context_lines, 5);
    // Non-overridden keys keep defaults
    assert_eq!(config.keybindings.toggle_file_tree, "ctrl+t");
}

#[test]
fn test_load_invalid_toml() {
    let toml = "this is not valid toml [[[";
    assert!(load_from_str(toml).is_err());
}

#[test]
fn test_partial_override_keeps_defaults() {
    let toml = r#"
[keybindings]
quit = "ctrl+q"
"#;
    let config = load_from_str(toml).unwrap();
    assert_eq!(config.keybindings.quit, "ctrl+q");
    assert_eq!(config.keybindings.push, "p");
    assert_eq!(config.keybindings.commit, "c");
    assert_eq!(config.diff.default_context_lines, 3);
}

#[test]
fn test_empty_toml_returns_defaults() {
    let config = load_from_str("").unwrap();
    assert_eq!(config.keybindings.quit, "q");
    assert_eq!(config.diff.default_context_lines, 3);
}
