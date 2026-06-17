use anyhow::{bail, Result};
use serde::Serialize;

const API_URL: &str = "https://api.pushover.net/1/messages.json";

#[derive(Debug, Serialize)]
pub struct Message<'a> {
    pub token: &'a str,
    pub user: &'a str,
    pub title: &'a str,
    pub message: &'a str,
    /// -2 (lowest) to 2 (emergency). Use 0 for normal, -1 for quiet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i8>,
    /// Pushover sound name, e.g. "pushover", "bike", "none".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound: Option<&'a str>,
}

#[derive(Debug, serde::Deserialize)]
struct PushoverResponse {
    status: i32,
    #[serde(default)]
    errors: Vec<String>,
}

pub async fn send(client: &reqwest::Client, msg: Message<'_>) -> Result<()> {
    let resp = client
        .post(API_URL)
        .form(&msg)
        .send()
        .await?
        .json::<PushoverResponse>()
        .await?;

    if resp.status != 1 {
        bail!("Pushover rejected message: {:?}", resp.errors);
    }

    Ok(())
}
