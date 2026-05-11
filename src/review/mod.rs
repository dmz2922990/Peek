use std::path::Path;

use anyhow::{Result, anyhow};

pub fn log(msg: &str) {
    use std::io::Write;
    let path = dirs_home().join("review.log");
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let _ = writeln!(f, "[{}] {}", ts, msg);
    }
}

fn dirs_home() -> std::path::PathBuf {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
}

use crate::app::state::{ReviewComment, ReviewSeverity};
use crate::config::types::ReviewConfig;
use crate::diff::types::{DiffLine, FileDiff};

const DEFAULT_REVIEW_PROMPT: &str = r#"You are an expert code reviewer.
1. The code under review is in diff format.
2. Analyze the changes and provide a thorough code review that includes:
- Analysis of code quality and style
- Specific suggestions for improvements
- Any potential issues or risks
- Any typo error"#;

const FORMAT_INSTRUCTION: &str = r#"

For each issue, output EXACTLY one line in this format:
L<line_number>:<ERROR|WARNING|SUGGESTION>:<concise summary>

If you need to add explanation, add a line starting with '>':
L42:ERROR:missing error handling for unwrap()
> The unwrap() on line 42 will panic if the Result is Err.

Rules:
- ERROR: bugs, security issues, crashes, data loss risks
- WARNING: code smells, potential bugs, missing edge case handling
- SUGGESTION: style improvements, readability, performance hints
- line_number refers to the NEW file line numbers (lines starting with +)
- Be specific and actionable
- If no issues found, respond with exactly: NO_ISSUES_FOUND
- Do NOT output anything else — no markdown, no headers, no explanations outside the format above"#;

pub fn get_prompt(config: &ReviewConfig) -> String {
    let base = match &config.prompt {
        Some(custom) if !custom.is_empty() => {
            format!("{}\n\n{}", DEFAULT_REVIEW_PROMPT, custom)
        }
        _ => DEFAULT_REVIEW_PROMPT.to_string(),
    };
    format!("{}{}", base, FORMAT_INSTRUCTION)
}

pub fn build_prompt(
    file: &FileDiff,
    source_lines: &[String],
    config: &ReviewConfig,
) -> String {
    let mut prompt = get_prompt(config);
    prompt.push_str("\n\n");

    let lang = &config.language;
    if lang != "en" {
        prompt.push_str(&format!("Respond in {}.\n\n", lang));
    }

    prompt.push_str(&format!("File: {}\n\n", file.new_path.display()));
    prompt.push_str("```diff\n");

    let full = config.context_lines.is_full();
    let ctx_n = config.context_lines.lines().unwrap_or(0);

    if full {
        for (i, line) in source_lines.iter().enumerate() {
            prompt.push_str(&format!(" {:>4} | {}\n", i + 1, line));
        }
        prompt.push_str("\n--- Diff hunks ---\n\n");
    }

    for (hunk_idx, hunk) in file.hunks.iter().enumerate() {
        prompt.push_str(&format!(
            "@@ -{},{} +{},{} @@{}\n",
            hunk.old_start,
            hunk.old_count,
            hunk.new_start,
            hunk.new_count,
            hunk.function_name.as_deref().map(|f| format!(" {}", f)).unwrap_or_default()
        ));

        if !full && ctx_n > 0 && hunk_idx == 0 {
            let start = hunk.new_start.saturating_sub(ctx_n).max(1);
            for line_no in start..hunk.new_start {
                if let Some(content) = source_lines.get(line_no.saturating_sub(1)) {
                    prompt.push_str(&format!(" {:>4} | {}\n", line_no, content));
                }
            }
        }

        for dl in &hunk.lines {
            match dl {
                DiffLine::Context { content, new_line, .. } => {
                    prompt.push_str(&format!(" {:>4} | {}\n", new_line, content));
                }
                DiffLine::Add { content, new_line } => {
                    prompt.push_str(&format!("+{:>4} | {}\n", new_line, content));
                }
                DiffLine::Delete { content, old_line } => {
                    prompt.push_str(&format!("-{:>4} | {}\n", old_line, content));
                }
            }
        }

        if !full && ctx_n > 0 {
            let after_start = hunk.new_start + hunk.new_count;
            for offset in 0..ctx_n {
                let line_no = after_start + offset;
                if let Some(content) = source_lines.get(line_no.saturating_sub(1)) {
                    prompt.push_str(&format!(" {:>4} | {}\n", line_no, content));
                }
            }
        }
    }

    prompt.push_str("```\n");
    prompt
}

pub async fn call_api(prompt: String, config: &ReviewConfig) -> Result<String> {
    let api_key = config.effective_api_key();
    if api_key.is_empty() {
        return Err(anyhow!("API key not configured. Set PEEK_API_KEY env or review.api_key in config."));
    }

    let base = config.base_url.trim_end_matches('/');
    let url = format!("{}/v1/messages", base);

    let client = reqwest::Client::new();
    let body = serde_json::json!({
        "model": config.model,
        "max_tokens": config.max_tokens,
        "messages": [{"role": "user", "content": prompt}]
    });

    let resp = client
        .post(&url)
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow!("API error ({}): {}", status, text));
    }

    let json: serde_json::Value = resp.json().await?;

    let content: String = json["content"]
        .as_array()
        .and_then(|arr| {
            arr.iter()
                .filter_map(|block| {
                    if block["type"].as_str() == Some("text") {
                        block["text"].as_str()
                    } else {
                        None
                    }
                })
                .next()
        })
        .unwrap_or("")
        .to_string();

    let stop_reason = json["stop_reason"].as_str().unwrap_or("");
    if content.is_empty() && stop_reason == "max_tokens" {
        return Err(anyhow!(
            "Model used all tokens for thinking. Try a non-thinking model or increase max_tokens."
        ));
    }

    Ok(content)
}

pub fn parse_response(body: &str) -> Vec<ReviewComment> {
    if body.trim() == "NO_ISSUES_FOUND" {
        return vec![ReviewComment {
            line_no: 0,
            severity: ReviewSeverity::Suggestion,
            summary: "No issues found. Code looks good!".into(),
            detail: String::new(),
        }];
    }

    let mut comments: Vec<ReviewComment> = Vec::new();
    let mut pending_detail = String::new();

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(detail_text) = trimmed.strip_prefix('>') {
            pending_detail.push_str(detail_text.trim());
            pending_detail.push('\n');
            continue;
        }

        if let Some(comment) = parse_comment_line(trimmed) {
            if !pending_detail.is_empty() && !comments.is_empty() {
                comments.last_mut().unwrap().detail = pending_detail.trim().to_string();
                pending_detail.clear();
            }
            comments.push(comment);
        } else {
            pending_detail.clear();
            if comments.is_empty() {
                comments.push(ReviewComment {
                    line_no: 0,
                    severity: ReviewSeverity::Suggestion,
                    summary: "General Review".into(),
                    detail: trimmed.to_string(),
                });
            }
        }
    }

    if !pending_detail.is_empty() && !comments.is_empty() {
        comments.last_mut().unwrap().detail = pending_detail.trim().to_string();
    }

    if comments.is_empty() {
        comments.push(ReviewComment {
            line_no: 0,
            severity: ReviewSeverity::Suggestion,
            summary: "No structured issues found".into(),
            detail: body.to_string(),
        });
    }

    comments
}

fn parse_comment_line(line: &str) -> Option<ReviewComment> {
    let line = line.strip_prefix('L')?;
    let colon1 = line.find(':')?;
    let rest = &line[colon1 + 1..];
    let colon2 = rest.find(':')?;

    let line_no: usize = line[..colon1].parse().ok()?;
    let severity_str = &rest[..colon2];
    let summary = rest[colon2 + 1..].to_string();

    let severity = match severity_str {
        "ERROR" => ReviewSeverity::Error,
        "WARNING" => ReviewSeverity::Warning,
        "SUGGESTION" => ReviewSeverity::Suggestion,
        _ => return None,
    };

    Some(ReviewComment {
        line_no,
        severity,
        summary,
        detail: String::new(),
    })
}

pub fn load_source_lines(path: &Path, repo_root: Option<&Path>) -> Vec<String> {
    let resolved = match repo_root {
        Some(root) => root.join(path),
        None => path.to_path_buf(),
    };
    std::fs::read_to_string(resolved)
        .map(|content| content.lines().map(String::from).collect())
        .unwrap_or_default()
}
