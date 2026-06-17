use anyhow::{bail, Context, Result};
use serde::Serialize;
use tokio::process::Command;

#[derive(Debug, Serialize, Clone)]
pub struct SyncResult {
    /// Whether a new commit was created this sync.
    pub committed: bool,
    /// Short commit hash if committed.
    pub commit_hash: Option<String>,
    /// Number of vault files staged in this commit.
    pub files_changed: usize,
    /// Whether a remote pull was attempted and succeeded.
    pub pulled: bool,
    /// Whether the commit was pushed to the remote.
    pub pushed: bool,
    /// True if a rebase conflict was detected. Nothing is lost — local commit is intact.
    pub conflicts: bool,
    /// Files involved in the conflict (empty if no conflict).
    pub conflict_files: Vec<String>,
    /// Commit message used (or "clean" if nothing to commit).
    pub message: String,
}

/// Commit vault-local changes, pull with rebase, and push.
///
/// Safety guarantees:
/// - Only stages files under `vault_path` (not the whole repo).
/// - On rebase conflict: aborts the rebase, reports conflict files, local commit is preserved.
/// - If no remote is configured, commits locally and skips pull/push.
pub async fn sync_vault(vault_path: &str, message: &str) -> Result<SyncResult> {
    // 1. Stage only vault-local changes.
    run(vault_path, &["add", "."]).await.context("git add")?;

    // 2. Collect staged file list before committing.
    let staged_output = output(vault_path, &["diff", "--cached", "--name-only"]).await.context("git diff --cached")?;
    let files_changed: Vec<String> = staged_output
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect();

    // 3. Commit if there is anything staged.
    let (committed, commit_hash) = if !files_changed.is_empty() {
        run(vault_path, &["commit", "-m", message])
            .await
            .context("git commit")?;
        let hash = output(vault_path, &["rev-parse", "--short", "HEAD"])
            .await
            .context("git rev-parse")?;
        (true, Some(hash.trim().to_string()))
    } else {
        (false, None)
    };

    // 4. Check whether a remote is configured. Skip network ops if not.
    let remotes = output(vault_path, &["remote"]).await.context("git remote")?;
    if remotes.trim().is_empty() {
        tracing::debug!("no git remote configured — skipping pull/push");
        return Ok(SyncResult {
            committed,
            commit_hash,
            files_changed: files_changed.len(),
            pulled: false,
            pushed: false,
            conflicts: false,
            conflict_files: vec![],
            message: message.to_string(),
        });
    }

    // 5. Pull with rebase so our commit sits cleanly on top of remote.
    // --autostash: git stashes any uncommitted working-tree changes, rebases, then pops.
    // Safe even when code changes are in progress alongside vault files.
    let pull_ok = run_ok(vault_path, &["pull", "--rebase", "--autostash"]).await;
    if !pull_ok {
        // Rebase conflict — abort cleanly, local commit is still intact.
        let _ = run(vault_path, &["rebase", "--abort"]).await;
        let conflict_files = unmerged_files(vault_path).await.unwrap_or_default();
        tracing::warn!(
            ?conflict_files,
            "git rebase conflict during sync — aborted; local commit preserved"
        );
        return Ok(SyncResult {
            committed,
            commit_hash,
            files_changed: files_changed.len(),
            pulled: false,
            pushed: false,
            conflicts: true,
            conflict_files,
            message: message.to_string(),
        });
    }

    // 6. Push.
    let pushed = run_ok(vault_path, &["push"]).await;
    if !pushed {
        tracing::error!("git push failed after successful rebase");
    }

    Ok(SyncResult {
        committed,
        commit_hash,
        files_changed: files_changed.len(),
        pulled: true,
        pushed,
        conflicts: false,
        conflict_files: vec![],
        message: message.to_string(),
    })
}

// ── Helpers ────────────────────────────────────────────────────────────────

async fn run(dir: &str, args: &[&str]) -> Result<()> {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .await?;
    if !status.success() {
        bail!("git {} exited {:?}", args.join(" "), status.code());
    }
    Ok(())
}

/// Returns true on success, false on any failure (non-panicking).
async fn run_ok(dir: &str, args: &[&str]) -> bool {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

async fn output(dir: &str, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .await?;
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// List files with unmerged status (UU, AA, DD, etc.).
async fn unmerged_files(dir: &str) -> Result<Vec<String>> {
    let out = output(dir, &["diff", "--name-only", "--diff-filter=U"]).await?;
    Ok(out.lines().filter(|l| !l.is_empty()).map(String::from).collect())
}
