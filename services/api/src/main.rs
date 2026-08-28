use std::{env, fs, io, net::SocketAddr, path::PathBuf};

use rand::Rng;
use reminder_proof_api::app;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

const BUILD_SHA: &str = match option_env!("BUILD_SHA") {
    Some(value) => value,
    None => "dev",
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);
    let dist_dir = env::var_os("DIST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("dist"));
    let (demo_secret, secret_source) = load_demo_secret()?;
    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(address).await?;

    info!(
        address = %address,
        build_sha = BUILD_SHA,
        dist_dir = %dist_dir.display(),
        demo_cookie_secret = secret_source,
        "Reminder Proof API started; demo cookie secret is supplied or generated without printing its value"
    );

    axum::serve(
        listener,
        app(BUILD_SHA, dist_dir, demo_secret).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

fn load_demo_secret() -> io::Result<(Vec<u8>, &'static str)> {
    if let Ok(value) = env::var("DEMO_COOKIE_SECRET") {
        let bytes = hex::decode(value).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "DEMO_COOKIE_SECRET must be hexadecimal",
            )
        })?;
        if bytes.len() < 32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "DEMO_COOKIE_SECRET must contain at least 32 bytes",
            ));
        }
        return Ok((bytes, "supplied"));
    }

    let data_dir = env::var_os("DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("data"));
    fs::create_dir_all(&data_dir)?;
    let path = data_dir.join("demo-cookie.key");
    if path.exists() {
        return Ok((fs::read(path)?, "persisted"));
    }
    let mut secret = vec![0_u8; 32];
    rand::rng().fill(&mut secret[..]);
    let temporary = data_dir.join(".demo-cookie.key.new");
    fs::write(&temporary, &secret)?;
    fs::rename(temporary, path)?;
    Ok((secret, "generated"))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
