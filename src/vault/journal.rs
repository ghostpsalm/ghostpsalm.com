use anyhow::Result;
use chrono::NaiveDate;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::db::models::HealthEntry;

/// Append a plain note to the daily journal file, creating it if needed.
pub async fn append_note(vault_root: &str, date: NaiveDate, note: &str) -> Result<String> {
    let path = daily_path(vault_root, date);
    fs::create_dir_all(path.parent().unwrap()).await?;

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await?;

    let entry = format!("\n## {}\n\n{}\n", chrono::Local::now().format("%H:%M"), note);
    file.write_all(entry.as_bytes()).await?;

    Ok(path.to_string_lossy().into_owned())
}

/// Append a structured health entry to the daily journal.
pub async fn append_health(vault_root: &str, entry: &HealthEntry) -> Result<()> {
    let note = format!(
        "**Health log**\n- Fatigue: {}\n- Sleep: {}h\n- Glucose: {}\n- Symptoms: {}\n- Notes: {}",
        entry.fatigue_rating.map(|v| v.to_string()).as_deref().unwrap_or("—"),
        entry.sleep_hours.map(|v| v.to_string()).as_deref().unwrap_or("—"),
        entry.glucose_readings.as_ref().map(|v| v.to_string()).as_deref().unwrap_or("—"),
        entry.symptoms.as_deref().unwrap_or("—"),
        entry.notes.as_deref().unwrap_or("—"),
    );
    let date = entry.date;
    append_note(vault_root, date, &note).await?;
    Ok(())
}

fn daily_path(vault_root: &str, date: NaiveDate) -> PathBuf {
    PathBuf::from(vault_root)
        .join("journal")
        .join(date.format("%Y").to_string())
        .join(format!("{}.md", date.format("%Y-%m-%d")))
}
