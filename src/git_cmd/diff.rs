use anyhow::{Result, anyhow};
use std::process::Command;

use crate::diff::types::DiffMode;

pub fn run_git_diff(mode: &DiffMode) -> Result<String> {
    match mode {
        DiffMode::Head => run_diff_head(),
        DiffMode::MainBranch => run_diff_branch("main"),
        DiffMode::OtherBranch(branch) => run_diff_branch(branch),
    }
}

fn run_diff_head() -> Result<String> {
    let output = Command::new("git")
        .args(["diff", "HEAD"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("git diff HEAD failed: {}", stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn run_diff_branch(branch: &str) -> Result<String> {
    // Fetch the branch first
    let fetch_output = Command::new("git")
        .args(["fetch", "origin", branch])
        .output()?;

    if !fetch_output.status.success() {
        let stderr = String::from_utf8_lossy(&fetch_output.stderr);
        return Err(anyhow!("git fetch origin {} failed: {}", branch, stderr));
    }

    // Find merge base
    let base_output = Command::new("git")
        .args(["merge-base", &format!("origin/{}", branch), "HEAD"])
        .output()?;

    if !base_output.status.success() {
        let stderr = String::from_utf8_lossy(&base_output.stderr);
        return Err(anyhow!("git merge-base failed: {}", stderr));
    }

    let base = String::from_utf8_lossy(&base_output.stdout).trim().to_string();

    // Diff against merge base
    let diff_output = Command::new("git")
        .args(["diff", &base, "HEAD"])
        .output()?;

    if !diff_output.status.success() {
        let stderr = String::from_utf8_lossy(&diff_output.stderr);
        return Err(anyhow!("git diff {} HEAD failed: {}", base, stderr));
    }

    Ok(String::from_utf8_lossy(&diff_output.stdout).into_owned())
}

pub fn current_branch() -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("failed to get current branch"));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn repo_root() -> Result<std::path::PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("failed to get repo root"));
    }

    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(std::path::PathBuf::from(path))
}

pub fn has_uncommitted_changes() -> Result<bool> {
    let output = Command::new("git")
        .args(["diff", "--quiet", "HEAD"])
        .output()?;

    // git diff --quiet exits with 1 if there are changes
    Ok(output.status.code() == Some(1))
}
