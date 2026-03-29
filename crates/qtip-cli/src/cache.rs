use std::path::{Path, PathBuf};

/// Returns `~/.qtip/cache/<safe-repo-name>/`
fn cache_dir_for(repo: &str) -> Result<PathBuf, String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "Could not determine home directory".to_string())?;

    let safe_name = repo.replace('/', "__");
    Ok(PathBuf::from(home).join(".qtip").join("cache").join(safe_name))
}

fn sha_file(cache_dir: &Path) -> PathBuf {
    cache_dir.join(".qtip-sha")
}

/// Query remote HEAD SHA without cloning.
fn remote_head_sha(repo_url: &str) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(["ls-remote", repo_url, "HEAD"])
        .output()
        .map_err(|e| format!("Failed to run git ls-remote: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git ls-remote failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .split_whitespace()
        .next()
        .map(|s| s.to_string())
        .ok_or_else(|| "git ls-remote returned no SHA".to_string())
}

fn cached_sha(cache_dir: &Path) -> Option<String> {
    std::fs::read_to_string(sha_file(cache_dir))
        .ok()
        .map(|s| s.trim().to_string())
}

fn write_sha(cache_dir: &Path, sha: &str) -> Result<(), String> {
    std::fs::write(sha_file(cache_dir), sha)
        .map_err(|e| format!("Failed to write SHA cache: {e}"))
}

fn clone_repo(repo_url: &str, dest: &Path) -> Result<(), String> {
    if dest.exists() {
        std::fs::remove_dir_all(dest)
            .map_err(|e| format!("Failed to clean cache dir: {e}"))?;
    }

    let output = std::process::Command::new("git")
        .args(["clone", "--depth", "1", repo_url, &dest.to_string_lossy()])
        .output()
        .map_err(|e| format!("Failed to run git clone: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git clone failed: {stderr}"));
    }

    Ok(())
}

fn pull_repo(cache_dir: &Path) -> Result<(), String> {
    let output = std::process::Command::new("git")
        .args(["fetch", "--depth", "1", "origin"])
        .current_dir(cache_dir)
        .output()
        .map_err(|e| format!("Failed to run git fetch: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git fetch failed: {stderr}"));
    }

    let output = std::process::Command::new("git")
        .args(["reset", "--hard", "origin/HEAD"])
        .current_dir(cache_dir)
        .output()
        .map_err(|e| format!("Failed to run git reset: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git reset failed: {stderr}"));
    }

    Ok(())
}

/// Ensure scenarios repo is cloned and up-to-date. Returns path to cached repo.
pub fn sync_repo(repo: &str) -> Result<PathBuf, String> {
    let repo_url = if repo.contains("://") || repo.starts_with("git@") {
        repo.to_string()
    } else {
        format!("https://github.com/{repo}.git")
    };

    let cache_dir = cache_dir_for(repo)?;

    let remote_sha = remote_head_sha(&repo_url)?;
    let local_sha = cached_sha(&cache_dir);

    if local_sha.as_deref() == Some(remote_sha.as_str()) && cache_dir.join(".git").exists() {
        eprintln!("Scenarios cache hit ({}) [{}]", repo, &remote_sha[..8]);
        return Ok(cache_dir);
    }

    if cache_dir.join(".git").exists() {
        eprintln!(
            "Scenarios cache stale ({}), pulling [{}]",
            repo,
            &remote_sha[..8]
        );
        pull_repo(&cache_dir)?;
    } else {
        eprintln!(
            "Scenarios cache miss ({}), cloning [{}]",
            repo,
            &remote_sha[..8]
        );
        std::fs::create_dir_all(cache_dir.parent().unwrap())
            .map_err(|e| format!("Failed to create cache dir: {e}"))?;
        clone_repo(&repo_url, &cache_dir)?;
    }

    write_sha(&cache_dir, &remote_sha)?;
    Ok(cache_dir)
}
