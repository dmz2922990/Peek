use crate::diff::types::{DiffLine, FileDiff, Hunk};

pub struct CopySelection {
    pub file: FileDiff,
    pub start_line: usize,
    pub end_line: usize,
    pub hunks: Vec<Hunk>,
}

pub fn format_smart_copy(file: &FileDiff, hunks: &[Hunk], start_line: usize, end_line: usize) -> String {
    let path = file.display_path().display();

    // Find function name from the first hunk that overlaps with the selection
    let func_name = hunks.iter()
        .find(|h| {
            let hunk_start = h.new_start;
            let hunk_end = h.new_start + h.new_count;
            hunk_start <= end_line && hunk_end >= start_line
        })
        .and_then(|h| h.function_name.clone());

    // Build header
    let header = match func_name {
        Some(func) => format!("{}:{}-{} → {}", path, start_line, end_line, func),
        None => format!("{}:{}-{}", path, start_line, end_line),
    };

    // Collect diff lines in range
    let mut diff_lines = Vec::new();
    for hunk in hunks {
        for line in &hunk.lines {
            match line {
                DiffLine::Context { content, new_line, .. } => {
                    if *new_line >= start_line && *new_line <= end_line {
                        diff_lines.push(format!(" {}", content));
                    }
                }
                DiffLine::Add { content, new_line } => {
                    if *new_line >= start_line && *new_line <= end_line {
                        diff_lines.push(format!("+{}", content));
                    }
                }
                DiffLine::Delete { content, old_line } => {
                    // Include deletions that are near the selection range
                    if *old_line >= start_line.saturating_sub(hunk.new_count) && *old_line <= end_line {
                        diff_lines.push(format!("-{}", content));
                    }
                }
            }
        }
    }

    // Build hunk header
    let hunk_header = format!("@@ -{},{} +{},{} @@", start_line, diff_lines.len(), start_line, diff_lines.len());

    format!(
        "{}\n\n```diff\n{}\n{}\n```",
        header,
        hunk_header,
        diff_lines.join("\n")
    )
}

pub fn format_single_line_copy(file: &FileDiff, hunk: &Hunk, line_idx: usize) -> String {
    let new_line = match hunk.lines.get(line_idx) {
        Some(DiffLine::Add { new_line, .. }) => *new_line,
        Some(DiffLine::Context { new_line, .. }) => *new_line,
        Some(DiffLine::Delete { old_line, .. }) => *old_line,
        None => return String::new(),
    };

    format_smart_copy(file, std::slice::from_ref(hunk), new_line, new_line)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::types::{DiffLine, FileStatus};
    use std::path::PathBuf;

    fn make_test_file() -> FileDiff {
        FileDiff {
            old_path: PathBuf::from("src/diff/parser.rs"),
            new_path: PathBuf::from("src/diff/parser.rs"),
            status: FileStatus::Modified,
            additions: 2,
            deletions: 1,
            is_binary: false,
            hunks: vec![Hunk {
                old_start: 42,
                old_count: 6,
                new_start: 42,
                new_count: 8,
                function_name: Some("fn parse_hunks()".into()),
                lines: vec![
                    DiffLine::Context { content: "let lines = content.lines();".into(), old_line: 42, new_line: 42 },
                    DiffLine::Context { content: "let result = parse(input);".into(), old_line: 43, new_line: 43 },
                    DiffLine::Delete { content: "let z = x + y;".into(), old_line: 44 },
                    DiffLine::Add { content: "let z = x * y;".into(), new_line: 44 },
                    DiffLine::Add { content: "validate_lines(&lines)?;".into(), new_line: 45 },
                    DiffLine::Context { content: "Ok(result)".into(), old_line: 46, new_line: 46 },
                ],
            }],
        }
    }

    #[test]
    fn test_smart_copy_with_function_name() {
        let file = make_test_file();
        let result = format_smart_copy(&file, &file.hunks, 42, 46);
        assert!(result.contains("src/diff/parser.rs:42-46 → fn parse_hunks()"));
        assert!(result.contains("```diff"));
        assert!(result.contains("+let z = x * y;"));
        assert!(result.contains("-let z = x + y;"));
    }

    #[test]
    fn test_smart_copy_single_line() {
        let file = make_test_file();
        let result = format_single_line_copy(&file, &file.hunks[0], 3);
        assert!(result.contains("src/diff/parser.rs"));
    }

    #[test]
    fn test_smart_copy_no_function_name() {
        let mut file = make_test_file();
        file.hunks[0].function_name = None;
        let result = format_smart_copy(&file, &file.hunks, 42, 46);
        assert!(result.contains("src/diff/parser.rs:42-46"));
        assert!(!result.contains("→"));
    }
}
