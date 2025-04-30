use crate::chat;
use crate::connection;
use crate::sdp;

use anyhow::Result;
use log::{error, info};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

// Application logic for the offerer role
pub async fn run_offerer(connection_timeout: Duration) -> Result<()> {
    info!("Starting as offerer...");
    println!(
        "Initializing connection with a timeout of {} seconds",
        connection_timeout.as_secs()
    );

    let start_time = Instant::now();

    // Create peer connection with TURN fallback support
    let pc = connection::create_peer_connection(true).await?;

    // Set up ICE candidate handling with improved buffering
    let candidates_mutex = Arc::new(Mutex::new(Vec::new()));
    let candidates_clone = Arc::clone(&candidates_mutex);

    pc.on_ice_candidate(Box::new(move |c| {
        if let Some(c) = c {
            match serde_json::to_string(&c) {
                Ok(candidate_str) => {
                    // Store the candidate for later use
                    let candidates = Arc::clone(&candidates_clone);
                    tokio::spawn(async move {
                        candidates.lock().await.push(candidate_str);
                    });
                }
                Err(e) => {
                    error!("Failed to serialize ICE candidate: {}", e);
                }
            }
        }
        Box::pin(async {})
    }));

    // Set up data channel
    let dc = connection::setup_data_channel(Arc::clone(&pc), "messaging", true).await?;

    // Generate and display the SDP offer
    sdp::generate_offer(&pc, &candidates_mutex).await?;

    // Read the answer from the peer
    println!("\nSend the above text to the answerer, then paste their response below:");
    println!("(Waiting for peer response...)\n");

    let response = sdp::read_sdp_input().await?;

    // Parse the answer
    let answer = sdp::parse_answer(&response)?;

    // Set the remote description
    match pc.set_remote_description(answer).await {
        Ok(_) => {
            println!("Remote description set successfully");
        }
        Err(e) => {
            error!("Error setting remote description: {}", e);
            return Err(anyhow::anyhow!("Failed to set remote description: {}", e));
        }
    }

    // Process ICE candidates from the peer
    sdp::process_ice_candidates(&response, &pc).await?;

    // Monitor connection state
    connection::monitor_connection_state(Arc::clone(&pc), connection_timeout, start_time).await?;

    // Start the chat session
    chat::enhanced_message_loop(dc).await?;

    Ok(())
}

// Application logic for the answerer role
pub async fn run_answerer(connection_timeout: Duration) -> Result<()> {
    info!("Starting as answerer...");
    println!(
        "Initializing connection with a timeout of {} seconds",
        connection_timeout.as_secs()
    );

    let start_time = Instant::now();

    // Create peer connection with TURN fallback support
    let pc = connection::create_peer_connection(true).await?;

    // Set up ICE candidate handling with improved buffering
    let candidates_mutex = Arc::new(Mutex::new(Vec::new()));
    let candidates_clone = Arc::clone(&candidates_mutex);

    pc.on_ice_candidate(Box::new(move |c| {
        if let Some(c) = c {
            match serde_json::to_string(&c) {
                Ok(candidate_str) => {
                    // Store the candidate for later use
                    let candidates = Arc::clone(&candidates_clone);
                    tokio::spawn(async move {
                        candidates.lock().await.push(candidate_str);
                    });
                }
                Err(e) => {
                    error!("Failed to serialize ICE candidate: {}", e);
                }
            }
        }
        Box::pin(async {})
    }));

    // Set up data channel (as answerer)
    let dc = connection::setup_data_channel(Arc::clone(&pc), "messaging", false).await?;

    // Read the offer from the peer
    let offer_data = sdp::read_sdp_input().await?;

    // Parse the offer
    let offer = sdp::parse_offer(&offer_data)?;

    // Set the remote description
    match pc.set_remote_description(offer).await {
        Ok(_) => {
            println!("Remote description set successfully");
        }
        Err(e) => {
            error!("Error setting remote description: {}", e);
            return Err(anyhow::anyhow!("Failed to set remote description: {}", e));
        }
    }

    // Process ICE candidates from the peer
    sdp::process_ice_candidates(&offer_data, &pc).await?;

    // Generate and display the SDP answer
    sdp::generate_answer(&pc, &candidates_mutex).await?;

    // Monitor connection state
    connection::monitor_connection_state(Arc::clone(&pc), connection_timeout, start_time).await?;

    // Start the chat session
    chat::enhanced_message_loop(dc).await?;

    Ok(())
}

// Placeholder for future group chat functionality
pub async fn run_group_chat(_max_peers: usize) -> Result<()> {
    println!("Group chat mode is experimental and not yet fully implemented");
    println!("It would support up to {} peers", _max_peers);
    Ok(())
}
