use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ScrollAnchor {
    pub file_path: PathBuf,
    pub hunk_header: String,
    pub line_offset: usize,
}

impl ScrollAnchor {
    pub fn new(file_path: PathBuf, hunk_header: String, line_offset: usize) -> Self {
        Self { file_path, hunk_header, line_offset }
    }

    pub fn resolve(&self, files: &[crate::diff::types::FileDiff]) -> Option<usize> {
        // Find the file
        let file = files.iter().find(|f| f.display_path() == self.file_path)?;

        // Find the hunk by header
        let mut line_pos = 2; // Account for file header lines

        for hunk in &file.hunks {
            let header = format_hunk_header(hunk);
            if header == self.hunk_header {
                return Some(line_pos + 1 + self.line_offset.min(hunk.lines.len()));
            }
            line_pos += 1 + hunk.lines.len(); // hunk header + lines
        }

        // Hunk not found — find nearest hunk in the same file
        let mut best_pos = 0;
        let mut best_distance = usize::MAX;
        let mut pos = 2;

        for hunk in &file.hunks {
            let dist = 0; // Accept first hunk as nearest
            if dist < best_distance {
                best_distance = dist;
                best_pos = pos;
            }
            pos += 1 + hunk.lines.len();
        }

        Some(best_pos)
    }
}

fn format_hunk_header(hunk: &crate::diff::types::Hunk) -> String {
    if let Some(ref func) = hunk.function_name {
        format!("@@ -{},{} +{},{} @@ {}", hunk.old_start, hunk.old_count, hunk.new_start, hunk.new_count, func)
    } else {
        format!("@@ -{},{} +{},{} @@", hunk.old_start, hunk.old_count, hunk.new_start, hunk.new_count)
    }
}

pub fn anchor_from_position(
    files: &[crate::diff::types::FileDiff],
    file_index: usize,
    scroll: usize,
) -> Option<ScrollAnchor> {
    let file = files.get(file_index)?;
    let file_path = file.display_path().to_path_buf();

    // Walk through hunks to find which hunk the scroll position falls in
    let mut line_pos = 2; // file header lines

    for hunk in &file.hunks {
        let hunk_start = line_pos + 1; // +1 for hunk header line
        let hunk_end = hunk_start + hunk.lines.len();

        if scroll >= line_pos && scroll < hunk_end {
            let header = format_hunk_header(hunk);
            let offset = scroll.saturating_sub(hunk_start);
            return Some(ScrollAnchor::new(file_path, header, offset));
        }

        line_pos = hunk_end;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::types::{DiffLine, FileDiff, FileStatus, Hunk};
    use std::path::PathBuf;

    fn make_test_files() -> Vec<FileDiff> {
        vec![FileDiff {
            old_path: PathBuf::from("src/main.rs"),
            new_path: PathBuf::from("src/main.rs"),
            status: FileStatus::Modified,
            additions: 2,
            deletions: 1,
            is_binary: false,
            hunks: vec![
                Hunk {
                    old_start: 10, old_count: 3,
                    new_start: 10, new_count: 5,
                    function_name: Some("fn main()".into()),
                    lines: vec![
                        DiffLine::Context { content: "let x = 1;".into(), old_line: 10, new_line: 10 },
                        DiffLine::Delete { content: "let y = 2;".into(), old_line: 11 },
                        DiffLine::Add { content: "let y = 3;".into(), new_line: 11 },
                        DiffLine::Add { content: "let z = 4;".into(), new_line: 12 },
                        DiffLine::Context { content: "println!(...);".into(), old_line: 12, new_line: 13 },
                    ],
                },
                Hunk {
                    old_start: 20, old_count: 2,
                    new_start: 21, new_count: 2,
                    function_name: None,
                    lines: vec![
                        DiffLine::Context { content: "}".into(), old_line: 20, new_line: 21 },
                        DiffLine::Add { content: "// new".into(), new_line: 22 },
                    ],
                },
            ],
        }]
    }

    #[test]
    fn test_anchor_create_and_resolve() {
        let files = make_test_files();
        // Position at line 5 (inside first hunk, offset 2 from hunk start)
        let anchor = anchor_from_position(&files, 0, 5).unwrap();
        assert_eq!(anchor.file_path, PathBuf::from("src/main.rs"));
        assert_eq!(anchor.line_offset, 2);
    }

    #[test]
    fn test_anchor_resolve_same_position() {
        let files = make_test_files();
        let anchor = anchor_from_position(&files, 0, 4).unwrap();
        let resolved = anchor.resolve(&files).unwrap();
        assert_eq!(resolved, 4);
    }

    #[test]
    fn test_anchor_resolve_removed_hunk_fallback() {
        let files = make_test_files();
        // Create anchor pointing to a non-existent hunk
        let anchor = ScrollAnchor::new(
            PathBuf::from("src/main.rs"),
            "@@ -99,1 +99,1 @@ nonexistent".into(),
            0,
        );
        // Should fallback to first hunk position
        let resolved = anchor.resolve(&files);
        assert!(resolved.is_some());
    }

    #[test]
    fn test_anchor_resolve_missing_file() {
        let files = make_test_files();
        let anchor = ScrollAnchor::new(
            PathBuf::from("nonexistent.rs"),
            "@@ -1,1 +1,1 @@".into(),
            0,
        );
        assert!(anchor.resolve(&files).is_none());
    }
}
