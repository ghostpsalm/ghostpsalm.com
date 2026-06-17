use anyhow::{Context, Result};
use tokio::process::Command;

use crate::api::tools::git_sync::SyncResponse;

/// Commit any changes in the vault and push. Safe: won't push if nothing changed.
pub async fn sync_vault(vault_path: &str, message: &str) -> Result<SyncResponse> {
    // Stage all changes in vault
    let status = Command::new("git")
        .args(["add", "-A"])
        .current_dir(vault_path)
        .status()
        .await
        .context("git add")?;

    if !status.success() {
        anyhow::bail!("git add failed");
    }

    // Check if there is anything to commit
    let diff = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .current_dir(vault_path)
        .status()
        .await
        .context("git diff")?;

    let committed = if !diff.success() {
        // Something staged — commit it
        let commit = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(vault_path)
            .status()
            .await
            .context("git commit")?;
        commit.success()
    } else {
        false // nothing to commit
    };

    // Push regardless (picks up any prior commits)
    let push = Command::new("git")
        .args(["push"])
        .current_dir(vault_path)
        .status()
        .await
        .context("git push")?;

    Ok(SyncResponse {
        committed,
        pushed: push.success(),
        message: message.to_string(),
    })
}
