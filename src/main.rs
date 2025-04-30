mod app;
mod chat;
mod cli;
mod connection;
mod sdp;

use anyhow::Result;
use clap::Parser;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize CLI first to get verbosity setting
    let cli = cli::Cli::parse();

    // Configure logging based on verbosity
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug,webrtc=debug");
    } else {
        std::env::set_var("RUST_LOG", "info,webrtc=warn");
    }
    env_logger::init();

    // Set up connection timeout from CLI
    let connection_timeout = Duration::from_secs(cli.timeout);

    // Execute the appropriate command
    match cli.command {
        cli::Commands::Offer => app::run_offerer(connection_timeout).await,
        cli::Commands::Answer => app::run_answerer(connection_timeout).await,
        cli::Commands::Group { max_peers } => app::run_group_chat(max_peers).await,
    }
}
