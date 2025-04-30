use clap::{Parser, Subcommand};

// CLI configuration
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Connection timeout in seconds
    #[arg(long, default_value = "60")]
    pub timeout: u64,

    /// Verbose logging
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start as offerer (initiates the connection)
    Offer,
    /// Start as answerer (waits for an offer)
    Answer,
    /// Create a group chat (experimental)
    #[command(hide = true)] // Hide this experimental feature
    Group {
        /// Maximum number of peers
        #[arg(short, long, default_value = "5")]
        max_peers: usize,
    },
}
