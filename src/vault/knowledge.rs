use anyhow::Result;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Write a knowledge note to the vault under `<vault_root>/<sub_path>/<slug>.md`.
pub async fn save_note(vault_root: &str, sub_path: &str, title: &str, content: &str) -> Result<String> {
    let slug = slugify(title);
    let dir = PathBuf::from(vault_root).join(sub_path);
    fs::create_dir_all(&dir).await?;

    let path = dir.join(format!("{}.md", slug));
    let frontmatter = format!(
        "---\ntitle: {}\ncreated: {}\n---\n\n",
        title,
        chrono::Local::now().format("%Y-%m-%dT%H:%M:%S")
    );

    let mut file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)
        .await?;
    file.write_all((frontmatter + content).as_bytes()).await?;

    Ok(path.to_string_lossy().into_owned())
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
