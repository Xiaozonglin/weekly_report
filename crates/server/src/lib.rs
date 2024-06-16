mod logging;
mod middleware;
mod routes;
mod traits;
use std::{net::SocketAddr, process};

use colored::Colorize;
use rustls::crypto;
use tracing::{error, info, warn};
use traits::GlobalState;
pub use traits::ResponseError;

/// Show greet information.
pub fn greet() {
    println!(
        "[START UP] {} {}",
        "XDSEC Weekly Report".bold(),
        format!(
            "{}-{}",
            env!("CARGO_PKG_VERSION"),
            git_version::git_version!(
                args = ["--abbrev=8", "--always", "--dirty=*"],
                fallback = "unknown"
            )
            .to_uppercase(),
        )
        .dimmed()
    );
    println!(
        "----------------------------- {} -----------------------------",
        "server log starts here".to_uppercase().bold()
    );
}

pub async fn up() -> anyhow::Result<()> {
    let console_guard = logging::initialize().await?;
    info!(">> Server initialization started <<");

    match crypto::aws_lc_rs::default_provider().install_default() {
        Ok(_) => info!("using `AWS Libcrypto` as default crypto backend."),
        Err(err) => {
            error!("`AWS Libcrypto` is not available: {:?}", err);
            warn!("try to use `ring` as default crypto backend.");
            crypto::ring::default_provider()
                .install_default()
                .inspect_err(|err| {
                    error!("`ring` is not available: {:?}", err);
                    error!("All crypto backend are not available, exiting...");
                    process::exit(1);
                })
                .ok();
            info!("using `ring` as default crypto backend.");
        }
    }
    info!("Loading module: < Database >");
    let db = wr_database::initialize().await?;

    let state = GlobalState {
        db,
        version: format!(
            "{}-{}",
            env!("CARGO_PKG_VERSION"),
            git_version::git_version!(
                args = ["--abbrev=8", "--always", "--dirty=*"],
                fallback = "unknown"
            )
            .to_uppercase()
        ),
    };
    info!("Modules loaded, constructing router...");

    let router = routes::initialize(state).await?;
    info!("Router constructed.");

    info!(">> Server initialization finished <<");

    info!("Starting server...");

    let addr_str = format!(
        "{}:{}",
        std::env::var("WR_HOST").unwrap(),
        std::env::var("WR_PORT").unwrap(),
    );

    let addr = tokio::net::TcpListener::bind(addr_str.clone())
        .await
        .expect("Failed to bind server address");
    info!("Server started at [ {} ]", addr_str);
    axum::serve(
        addr,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Failed to start server.");

    drop(console_guard);
    Ok(())
}
