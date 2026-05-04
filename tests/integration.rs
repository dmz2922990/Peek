use std::fs;
use std::process::Command;
use std::path::PathBuf;

use peek::diff::parser;
use peek::diff::types::{DiffLine, FileStatus};
use peek::clipboard::smart_copy;
use peek::config;
use peek::scroll;
use peek::app;

fn create_temp_repo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path();

    Command::new("git").args(["init"]).current_dir(path).output().unwrap();
    Command::new("git").args(["config", "user.email", "test@test.com"]).current_dir(path).output().unwrap();
    Command::new("git").args(["config", "user.name", "Test"]).current_dir(path).output().unwrap();

    // Create initial commit
    fs::write(path.join("hello.rs"), "fn main() {\n    println!(\"hello\");\n}\n").unwrap();
    fs::write(path.join("utils.rs"), "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n").unwrap();
    Command::new("git").args(["add", "."]).current_dir(path).output().unwrap();
    Command::new("git").args(["commit", "-m", "initial"]).current_dir(path).output().unwrap();

    dir
}

fn make_changes(repo_path: &PathBuf) {
    // Modify existing file
    fs::write(repo_path.join("hello.rs"),
        "fn main() {\n    println!(\"hello, world!\");\n    let x = utils::add(1, 2);\n    println!(\"{}\", x);\n}\n"
    ).unwrap();

    // Add new file and stage it so it appears in diff
    fs::write(repo_path.join("new_file.rs"),
        "pub fn greet(name: &str) -> String {\n    format!(\"Hello, {}!\", name)\n}\n"
    ).unwrap();
    Command::new("git").args(["add", "new_file.rs"]).current_dir(repo_path).output().unwrap();
}

#[test]
fn test_full_diff_pipeline() {
    let dir = create_temp_repo();
    let path = dir.path().to_path_buf();
    make_changes(&path);

    // Run git diff
    let output = Command::new("git")
        .args(["diff", "HEAD"])
        .current_dir(&path)
        .output()
        .unwrap();
    let diff_stdout = String::from_utf8_lossy(&output.stdout);

    println!("=== Raw git diff output ===");
    println!("{}", diff_stdout);
    println!("===========================");

    // Parse
    let files = parser::parse_diff(&diff_stdout).unwrap();
    println!("\n=== Parsed files: {} ===", files.len());
    for f in &files {
        println!("  {} {:?} +{} -{} hunks:{}", f.display_path().display(), f.status, f.additions, f.deletions, f.hunks.len());
    }

    // Verify file count and statuses
    assert!(files.len() >= 2, "Expected at least 2 files, got {}", files.len());

    let hello_file = files.iter().find(|f| f.new_path.to_string_lossy().contains("hello.rs")).unwrap();
    assert_eq!(hello_file.status, FileStatus::Modified);
    assert!(hello_file.additions > 0);
    assert!(hello_file.deletions > 0);

    let new_file = files.iter().find(|f| f.new_path.to_string_lossy().contains("new_file.rs")).unwrap();
    assert_eq!(new_file.status, FileStatus::Added);

    // Verify hunk parsing with function names
    println!("\n=== Hunk details for hello.rs ===");
    for (i, h) in hello_file.hunks.iter().enumerate() {
        println!("  hunk {}: @@ -{},{} +{},{} @@ func={:?}", i, h.old_start, h.old_count, h.new_start, h.new_count, h.function_name);
        for (j, line) in h.lines.iter().enumerate() {
            match line {
                DiffLine::Context { content, old_line, new_line } =>
                    println!("    [{}] {} {:>4} {:>4} {}", j, " ", old_line, new_line, content),
                DiffLine::Add { content, new_line } =>
                    println!("    [{}] {}      {:>4} {}", j, "+", new_line, content),
                DiffLine::Delete { content, old_line } =>
                    println!("    [{}] {} {:>4}     {}", j, "-", old_line, content),
            }
        }
    }
}

#[test]
fn test_smart_copy_output() {
    let dir = create_temp_repo();
    let path = dir.path().to_path_buf();
    make_changes(&path);

    let output = Command::new("git")
        .args(["diff", "HEAD"])
        .current_dir(&path)
        .output()
        .unwrap();
    let files = parser::parse_diff(&String::from_utf8_lossy(&output.stdout)).unwrap();

    let hello_file = files.iter().find(|f| f.new_path.to_string_lossy().contains("hello.rs")).unwrap();
    let hunk = &hello_file.hunks[0];

    let result = smart_copy::format_smart_copy(hello_file, &hello_file.hunks, hunk.new_start, hunk.new_start + hunk.new_count);

    println!("=== Smart Copy Output ===");
    println!("{}", result);
    println!("=========================");

    assert!(result.contains("hello.rs"));
    assert!(result.contains("```diff"));
}

#[test]
fn test_config_system() {
    let custom_toml = r#"
[keybindings]
quit = "Q"
copy = "Y"
push = "P"

[diff]
default_context_lines = 5
file_tree_width_percent = 40
"#;

    let config = config::load_from_str(custom_toml).unwrap();
    assert_eq!(config.keybindings.quit, "Q");
    assert_eq!(config.keybindings.copy, "Y");
    assert_eq!(config.keybindings.push, "P");
    assert_eq!(config.keybindings.commit, "c"); // default preserved
    assert_eq!(config.diff.default_context_lines, 5);
    assert_eq!(config.diff.file_tree_width_percent, 40);

    println!("=== Config loaded ===");
    println!("  quit: {}", config.keybindings.quit);
    println!("  copy: {}", config.keybindings.copy);
    println!("  commit: {} (default)", config.keybindings.commit);
    println!("  context_lines: {}", config.diff.default_context_lines);
}

#[test]
fn test_scroll_anchor() {
    let dir = create_temp_repo();
    let path = dir.path().to_path_buf();
    make_changes(&path);

    let output = Command::new("git")
        .args(["diff", "HEAD"])
        .current_dir(&path)
        .output()
        .unwrap();
    let files = parser::parse_diff(&String::from_utf8_lossy(&output.stdout)).unwrap();

    // Create anchor at position 4 in first file
    let anchor = scroll::anchor_from_position(&files, 0, 4).unwrap();
    println!("=== Scroll Anchor ===");
    println!("  file: {:?}", anchor.file_path);
    println!("  hunk_header: {}", anchor.hunk_header);
    println!("  offset: {}", anchor.line_offset);

    // Resolve back
    let resolved = anchor.resolve(&files).unwrap();
    assert_eq!(resolved, 4);
    println!("  resolved: {} (matches original)", resolved);
}

#[test]
fn test_git_operations() {
    let dir = create_temp_repo();
    let path = dir.path().to_path_buf();
    make_changes(&path);

    // Test git status
    let status = peek::git_cmd::ops::git_status_porcelain().unwrap();
    println!("=== Git Status (current repo) ===");
    for (s, p) in &status {
        println!("  {} {}", s, p);
    }

    // Test current branch
    let branch = peek::git_cmd::diff::current_branch().unwrap();
    println!("  current branch: {}", branch);

    // Test has uncommitted changes
    let has_changes = peek::git_cmd::diff::has_uncommitted_changes().unwrap();
    println!("  has uncommitted changes: {}", has_changes);
}

#[test]
fn test_app_state_transitions() {
    let config = config::load_from_str("").unwrap();
    let mut app = app::state::App::new(config);

    println!("=== App State Transitions ===");
    println!("  initial mode: {:?}", app.mode);

    assert_eq!(app.mode, app::state::AppMode::DiffViewFocus);

    // Simulate mode transitions
    app.file_tree.visible = true;
    println!("  file_tree.visible: {}", app.file_tree.visible);

    app.diff_view.extra_context = 6;
    println!("  extra_context: {}", app.diff_view.extra_context);

    app.diff_view.status_message = Some("test message".into());
    println!("  status_message: {:?}", app.diff_view.status_message);
}
