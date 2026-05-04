use anyhow::{Result, anyhow};
use std::process::Command;

pub fn git_commit(message: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("git commit failed: {}", stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn git_push(remote: &str, branch: &str, set_upstream: bool) -> Result<String> {
    let mut args = vec!["push".to_string()];
    if set_upstream {
        args.push("-u".to_string());
    }
    args.push(remote.to_string());
    args.push(branch.to_string());

    let output = Command::new("git")
        .args(&args)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("git push failed: {}", stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn git_remote_url() -> Result<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("no remote 'origin' configured"));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn git_status_porcelain() -> Result<Vec<(String, String)>> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()?;

    if !output.status.success() {
        return Err(anyhow!("git status failed"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let entries = stdout
        .lines()
        .filter_map(|line| {
            if line.len() >= 4 {
                let status = line[..2].trim().to_string();
                let path = line[3..].to_string();
                Some((status, path))
            } else {
                None
            }
        })
        .collect();

    Ok(entries)
}

pub fn git_log_oneline(count: usize, range: Option<&str>) -> Result<Vec<String>> {
    let count_arg = format!("-{}", count);
    let mut args: Vec<&str> = vec!["log", &count_arg, "--oneline"];
    if let Some(r) = range {
        args.push(r);
    }

    let output = Command::new("git").args(&args).output()?;

    if !output.status.success() {
        return Err(anyhow!("git log failed"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().map(String::from).collect())
}

pub fn has_upstream(branch: &str) -> Result<bool> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", &format!("{}@{{upstream}}", branch)])
        .output()?;

    Ok(output.status.success())
}

pub fn construct_pr_url(remote_url: &str, branch: &str) -> Option<String> {
    // Convert git@github.com:user/repo.git → https://github.com/user/repo
    let https_url = if remote_url.starts_with("git@") {
        let parts: Vec<&str> = remote_url.splitn(2, ':').collect();
        if parts.len() != 2 { return None; }
        let path = parts[1].trim_end_matches(".git");
        format!("https://github.com/{}", path)
    } else if remote_url.starts_with("https://") || remote_url.starts_with("http://") {
        remote_url.trim_end_matches(".git").to_string()
    } else {
        return None;
    };

    Some(format!("{}/compare/{}", https_url, branch))
}

pub fn open_url(url: &str) -> Result<()> {
    let (cmd, args) = if cfg!(target_os = "macos") {
        ("open", vec![url])
    } else if cfg!(target_os = "linux") {
        ("xdg-open", vec![url])
    } else {
        ("cmd", vec!["/C", "start", url])
    };

    Command::new(cmd).args(&args).output()?;
    Ok(())
}
