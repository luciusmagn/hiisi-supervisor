mod monitor;
mod ports;
mod process;
mod server;
mod state;

use server::Server;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .init();

    // Ensure we're running as root
    if !nix::unistd::Uid::effective().is_root() {
        error!("hiidet must be run as root");
        std::process::exit(1);
    }

    info!("Starting hiidet daemon");

    // Create socket directory if it doesn't exist
    std::fs::create_dir_all("/run/hiisi")?;

    let socket_path =
        Arc::new(PathBuf::from("/run/hiisi/hiisi.sock"));
    let socket_path_cleanup = Arc::clone(&socket_path);
    let server = Server::new();

    // Handle SIGTERM gracefully
    let (tx, rx) = tokio::sync::oneshot::channel();
    ctrlc::set_handler({
        let tx = std::sync::Mutex::new(Some(tx));
        move || {
            if let Some(tx) = tx.lock().unwrap().take() {
                tx.send(()).ok();
            }
        }
    })?;

    // Run server in background task
    let server_handle = tokio::spawn({
        let socket_path = Arc::clone(&socket_path);
        async move {
            if let Err(e) = server.run(&socket_path).await {
                error!("Server error: {}", e);
            }
        }
    });

    // Wait for shutdown signal
    rx.await?;
    info!("Shutdown signal received");

    // Clean up
    server_handle.abort();
    if socket_path_cleanup.exists() {
        std::fs::remove_file(&*socket_path_cleanup)?;
    }

    info!("Daemon stopped");
    Ok(())
}
