use anyhow::{Result, anyhow};
use std::path::PathBuf;

use super::types::{DiffLine, FileDiff, FileStatus, Hunk};

pub fn parse_diff(input: &str) -> Result<Vec<FileDiff>> {
    if input.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    let mut lines = input.lines().peekable();

    while lines.peek().is_some() {
        let file = parse_file(&mut lines)?;
        files.push(file);
    }

    Ok(files)
}

fn parse_file<'a, I: Iterator<Item = &'a str>>(mut lines: &mut std::iter::Peekable<I>) -> Result<FileDiff> {
    let mut old_path = PathBuf::new();
    let mut new_path = PathBuf::new();
    let mut status = FileStatus::Modified;
    let mut is_binary = false;

    // Parse file header
    loop {
        let line = lines.peek().ok_or_else(|| anyhow!("unexpected end of diff"))?;

        if line.starts_with("diff --git") {
            lines.next();
            continue;
        }

        if let Some(rest) = line.strip_prefix("--- ") {
            let p = rest.trim();
            old_path = if p == "/dev/null" {
                PathBuf::new()
            } else {
                PathBuf::from(p.strip_prefix("a/").unwrap_or(p))
            };
            lines.next();
            continue;
        }

        if let Some(rest) = line.strip_prefix("+++ ") {
            let p = rest.trim();
            new_path = if p == "/dev/null" {
                PathBuf::new()
            } else {
                PathBuf::from(p.strip_prefix("b/").unwrap_or(p))
            };
            lines.next();
            continue;
        }

        if line.starts_with("old mode") || line.starts_with("new mode") {
            lines.next();
            continue;
        }

        if let Some(rest) = line.strip_prefix("rename from ") {
            old_path = PathBuf::from(rest.trim());
            status = FileStatus::Renamed;
            lines.next();
            continue;
        }

        if let Some(rest) = line.strip_prefix("rename to ") {
            new_path = PathBuf::from(rest.trim());
            lines.next();
            continue;
        }

        if line.starts_with("copy from") || line.starts_with("copy to") {
            if line.starts_with("copy from") {
                status = FileStatus::Copied;
            }
            lines.next();
            continue;
        }

        if line.contains("Binary files") && line.contains("differ") {
            is_binary = true;
            lines.next();
            break;
        }

        // Hunks start with @@
        if line.starts_with("@@") {
            break;
        }

        // Skip other header lines (index, similarity index, etc.)
        lines.next();
    }

    // Determine status from paths
    if old_path.as_os_str().is_empty() && !new_path.as_os_str().is_empty() {
        status = FileStatus::Added;
    } else if new_path.as_os_str().is_empty() && !old_path.as_os_str().is_empty() {
        status = FileStatus::Deleted;
    }

    // Parse hunks
    let mut hunks = Vec::new();
    while lines.peek().map_or(false, |l| l.starts_with("@@")) {
        let hunk = parse_hunk(&mut lines)?;
        hunks.push(hunk);
    }

    let additions = hunks.iter().flat_map(|h| h.lines.iter())
        .filter(|l| matches!(l, DiffLine::Add { .. }))
        .count();
    let deletions = hunks.iter().flat_map(|h| h.lines.iter())
        .filter(|l| matches!(l, DiffLine::Delete { .. }))
        .count();

    Ok(FileDiff {
        old_path,
        new_path,
        status,
        hunks,
        additions,
        deletions,
        is_binary,
    })
}

fn parse_hunk<'a, I: Iterator<Item = &'a str>>(lines: &mut std::iter::Peekable<I>) -> Result<Hunk> {
    let header = lines.next().ok_or_else(|| anyhow!("expected hunk header"))?;

    // Parse @@ -old_start,old_count +new_start,new_count @@ [function_name]
    let (range_part, func_name) = split_hunk_header(header)?;

    let (old_start, old_count) = parse_range(&range_part.old)?;
    let (new_start, new_count) = parse_range(&range_part.new_)?;

    let mut hunk_lines = Vec::new();
    let mut old_line = old_start;
    let mut new_line = new_start;

    while let Some(&line) = lines.peek() {
        if line.starts_with("@@") || line.starts_with("diff --git") {
            break;
        }

        lines.next();

        if let Some(content) = line.strip_prefix('+') {
            hunk_lines.push(DiffLine::Add {
                content: content.to_string(),
                new_line,
            });
            new_line += 1;
        } else if let Some(content) = line.strip_prefix('-') {
            hunk_lines.push(DiffLine::Delete {
                content: content.to_string(),
                old_line,
            });
            old_line += 1;
        } else if let Some(content) = line.strip_prefix(' ') {
            hunk_lines.push(DiffLine::Context {
                content: content.to_string(),
                old_line,
                new_line,
            });
            old_line += 1;
            new_line += 1;
        } else if line.starts_with('\\') {
            // "\ No newline at end of file" — skip
            continue;
        } else {
            // Treat as context for bare lines
            hunk_lines.push(DiffLine::Context {
                content: line.to_string(),
                old_line,
                new_line,
            });
            old_line += 1;
            new_line += 1;
        }
    }

    Ok(Hunk {
        old_start,
        old_count,
        new_start,
        new_count,
        function_name: func_name,
        lines: hunk_lines,
    })
}

struct HunkRanges {
    old: String,
    new_: String,
}

fn split_hunk_header(header: &str) -> Result<(HunkRanges, Option<String>)> {
    // @@ -1,3 +1,4 @@ fn foo()
    let header = header.trim_start_matches('@');
    let header = header.trim_start_matches(' ');
    let header = header.trim_start_matches('-');

    // Split into "-1,3 +1,4" and rest
    let parts: Vec<&str> = header.splitn(2, " +").collect();
    if parts.len() < 2 {
        return Err(anyhow!("invalid hunk header: {}", header));
    }

    let old_part = parts[0].to_string();
    let rest = parts[1];

    // Split "+1,4 @@" or "+1,4 @@ fn foo()"
    let (new_part, func_name) = if let Some(idx) = rest.find("@@") {
        let new_p = rest[..idx].trim().to_string();
        let func = rest[idx + 2..].trim();
        let func = if func.is_empty() { None } else { Some(func.to_string()) };
        (new_p, func)
    } else {
        (rest.trim().to_string(), None)
    };

    Ok((HunkRanges { old: old_part, new_: new_part }, func_name))
}

fn parse_range(range: &str) -> Result<(usize, usize)> {
    let parts: Vec<&str> = range.split(',').collect();
    let start: usize = parts[0].parse().map_err(|_| anyhow!("invalid range start: {}", range))?;
    let count: usize = if parts.len() > 1 {
        parts[1].parse().map_err(|_| anyhow!("invalid range count: {}", range))?
    } else {
        1
    };
    Ok((start, count))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let result = parse_diff("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_single_modified_file() {
        let input = r#"diff --git a/src/main.rs b/src/main.rs
index abc123..def456 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -10,6 +10,8 @@ fn main() {
 let x = 1;
 let y = 2;
-let z = x + y;
+let z = x * y;
+println!("{}", z);
 let w = 3;
"#;
        let files = parse_diff(input).unwrap();
        assert_eq!(files.len(), 1);

        let f = &files[0];
        assert_eq!(f.new_path, PathBuf::from("src/main.rs"));
        assert_eq!(f.status, FileStatus::Modified);
        assert_eq!(f.hunks.len(), 1);
        assert_eq!(f.additions, 2);
        assert_eq!(f.deletions, 1);

        let h = &f.hunks[0];
        assert_eq!(h.old_start, 10);
        assert_eq!(h.old_count, 6);
        assert_eq!(h.new_start, 10);
        assert_eq!(h.new_count, 8);
        assert_eq!(h.function_name.as_deref(), Some("fn main() {"));
    }

    #[test]
    fn test_parse_added_file() {
        let input = r#"diff --git a/new_file.rs b/new_file.rs
new file mode 100644
--- /dev/null
+++ b/new_file.rs
@@ -0,0 +1,3 @@
+fn hello() {
+    println!("hello");
+}
"#;
        let files = parse_diff(input).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].status, FileStatus::Added);
        assert_eq!(files[0].additions, 3);
        assert_eq!(files[0].deletions, 0);
    }

    #[test]
    fn test_parse_deleted_file() {
        let input = r#"diff --git a/old_file.rs b/old_file.rs
deleted file mode 100644
--- a/old_file.rs
+++ /dev/null
@@ -1,2 +0,0 @@
-fn old() {}
-// removed
"#;
        let files = parse_diff(input).unwrap();
        assert_eq!(files[0].status, FileStatus::Deleted);
    }

    #[test]
    fn test_parse_renamed_file() {
        let input = r#"diff --git a/old.rs b/new.rs
similarity index 95%
rename from old.rs
rename to new.rs
--- a/old.rs
+++ b/new.rs
@@ -1,3 +1,3 @@
 fn same() {
-    old_code
+    new_code
 }
"#;
        let files = parse_diff(input).unwrap();
        assert_eq!(files[0].status, FileStatus::Renamed);
        assert_eq!(files[0].old_path, PathBuf::from("old.rs"));
        assert_eq!(files[0].new_path, PathBuf::from("new.rs"));
    }

    #[test]
    fn test_parse_binary_file() {
        let input = r#"diff --git a/image.png b/image.png
Binary files /dev/null and b/image.png differ
"#;
        let files = parse_diff(input).unwrap();
        assert_eq!(files[0].is_binary, true);
        assert!(files[0].hunks.is_empty());
    }

    #[test]
    fn test_hunk_header_with_function() {
        let header = "@@ -42,6 +42,8 @@ fn parse_hunks() -> Vec<Hunk> {";
        let (ranges, func) = split_hunk_header(header).unwrap();
        assert_eq!(ranges.old, "42,6");
        assert_eq!(ranges.new_, "42,8");
        assert_eq!(func.as_deref(), Some("fn parse_hunks() -> Vec<Hunk> {"));
    }

    #[test]
    fn test_hunk_header_without_function() {
        let header = "@@ -1,3 +1,4 @@";
        let (_, func) = split_hunk_header(header).unwrap();
        assert_eq!(func, None);
    }

    #[test]
    fn test_parse_multiple_files() {
        let input = r#"diff --git a/a.rs b/a.rs
new file mode 100644
--- /dev/null
+++ b/a.rs
@@ -0,0 +1,1 @@
+fn a() {}
diff --git a/b.rs b/b.rs
--- a/b.rs
+++ b/b.rs
@@ -1,2 +1,2 @@
 fn b() {
-    old
+    new
 }
diff --git a/c.rs b/c.rs
deleted file mode 100644
--- a/c.rs
+++ /dev/null
@@ -1,1 +0,0 @@
-fn c() {}
"#;
        let files = parse_diff(input).unwrap();
        assert_eq!(files.len(), 3);
        assert_eq!(files[0].status, FileStatus::Added);
        assert_eq!(files[1].status, FileStatus::Modified);
        assert_eq!(files[2].status, FileStatus::Deleted);
    }
}
