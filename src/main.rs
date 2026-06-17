mod api;
mod config;
mod db;
mod git;
mod notifications;
mod state;
mod vault;

use anyhow::{Context, Result};
use jsonwebtoken::{DecodingKey, EncodingKey};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ghostpsalm=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cfg = config::Config::load()?;
    let db_pool = db::connect(&cfg.database_url).await?;

    let private_pem = std::fs::read(&cfg.private_key_path)
        .with_context(|| format!("reading private key: {}", cfg.private_key_path))?;
    let public_pem = std::fs::read(&cfg.public_key_path)
        .with_context(|| format!("reading public key: {}", cfg.public_key_path))?;

    let encoding_key = EncodingKey::from_ed_pem(&private_pem)
        .context("parsing Ed25519 private key — ensure PKCS8 PEM format")?;
    let decoding_key = DecodingKey::from_ed_pem(&public_pem)
        .context("parsing Ed25519 public key — ensure SPKI PEM format")?;

    let state = AppState::new(db_pool, cfg.clone(), encoding_key, decoding_key);

    let _scheduler = notifications::start_scheduler(state.clone())
        .await
        .context("starting notification scheduler")?;

    let app = api::router(state);

    if cfg.tls_enabled() {
        let cert = cfg.tls_cert_path.as_deref().unwrap();
        let key = cfg.tls_key_path.as_deref().unwrap();
        let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(cert, key)
            .await
            .context("loading TLS certificate")?;
        let addr: std::net::SocketAddr = cfg.bind_addr.parse()
            .context("parsing BIND_ADDR")?;
        tracing::info!(addr = %cfg.bind_addr, "HTTPS listening");
        axum_server::bind_rustls(addr, tls_config)
            .serve(app.into_make_service())
            .await?;
    } else {
        let listener = tokio::net::TcpListener::bind(&cfg.bind_addr).await?;
        tracing::info!(addr = %cfg.bind_addr, tls = false, "HTTP listening");
        axum::serve(listener, app).await?;
    }

    Ok(())
}
