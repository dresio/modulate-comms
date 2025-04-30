use anyhow::{Context, Result};
use log::{debug, error, warn};
use serde_json;
use std::io::{self, BufRead};
use std::sync::Arc;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;

// Generate and display SDP offer
pub async fn generate_offer(
    pc: &Arc<RTCPeerConnection>,
    candidates_mutex: &Arc<tokio::sync::Mutex<Vec<String>>>,
) -> Result<()> {
    // Create an offer with additional configuration for better compatibility
    let offer = pc
        .create_offer(None)
        .await
        .context("Failed to create offer")?;
    pc.set_local_description(offer.clone())
        .await
        .context("Failed to set local description")?;

    // Wait for ICE gathering to complete or timeout
    wait_for_ice_gathering(candidates_mutex).await;

    // Generate offer with improved formatting for better copy/paste experience
    let offer_json = serde_json::to_string(&offer).context("Failed to serialize offer")?;

    // Color coded output for better visibility (if the terminal supports it)
    println!("\n==== COPY EVERYTHING BETWEEN THESE LINES ====");
    println!("OFFER:{}", offer_json);

    // Collect ICE candidates in a single string
    let pending_candidates = candidates_mutex.lock().await;
    println!("ICE_CANDIDATES:");
    println!(
        "{}",
        serde_json::to_string(&*pending_candidates)
            .context("Failed to serialize ICE candidates")?
    );
    println!("==== END OF SECTION TO COPY ====\n");

    Ok(())
}

// Generate and display SDP answer
pub async fn generate_answer(
    pc: &Arc<RTCPeerConnection>,
    candidates_mutex: &Arc<tokio::sync::Mutex<Vec<String>>>,
) -> Result<()> {
    // Create answer with additional configuration for better compatibility
    let answer = pc
        .create_answer(None)
        .await
        .context("Failed to create answer")?;

    // Fixed: Clone the answer before setting local description
    pc.set_local_description(answer.clone())
        .await
        .context("Failed to set local description")?;

    // Wait for ICE gathering to complete or timeout
    wait_for_ice_gathering(candidates_mutex).await;

    // Generate answer with improved formatting for better copy/paste experience
    let answer_json = serde_json::to_string(&answer).context("Failed to serialize answer")?;

    // Color coded output for better visibility (if the terminal supports it)
    println!("\n==== COPY EVERYTHING BETWEEN THESE LINES ====");
    println!("ANSWER:{}", answer_json);

    // Collect ICE candidates in a single string
    let pending_candidates = candidates_mutex.lock().await;
    println!("ICE_CANDIDATES:");
    println!(
        "{}",
        serde_json::to_string(&*pending_candidates)
            .context("Failed to serialize ICE candidates")?
    );
    println!("==== END OF SECTION TO COPY ====\n");

    Ok(())
}

// Wait for ICE gathering to complete
async fn wait_for_ice_gathering(candidates_mutex: &Arc<tokio::sync::Mutex<Vec<String>>>) -> usize {
    println!("Gathering ICE candidates (this may take a few seconds)...");

    // Wait for ICE gathering to complete or timeout
    let mut gathered_count = 0;
    let mut prev_count = 0;

    // Wait up to 5 seconds for initial candidates
    for _ in 0..10 {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let count = candidates_mutex.lock().await.len();
        if count > 0 && count == prev_count {
            // If no new candidates for a while, assume gathering is complete
            break;
        }
        prev_count = count;
        gathered_count = count;
    }

    println!("Gathered {} ICE candidates", gathered_count);
    gathered_count
}

// Read and parse offer or answer from user input
pub async fn read_sdp_input() -> Result<String> {
    println!("\nPaste everything from the other peer between the lines (including the lines):");
    println!("(Waiting for SDP data...)\n");

    // Improved input handling with better error recovery
    let mut sdp_data = String::new();
    let stdin = io::stdin();

    loop {
        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(_) => {}
            Err(e) => {
                error!("Error reading input: {}", e);
                continue;
            }
        }

        if line.contains("END OF SECTION") {
            sdp_data.push_str(&line);
            break;
        }

        if !line.trim().is_empty() {
            sdp_data.push_str(&line);
        }
    }

    Ok(sdp_data)
}

// Parse SDP offer from input text
pub fn parse_offer(offer_data: &str) -> Result<RTCSessionDescription> {
    let offer_line = offer_data
        .lines()
        .find(|line| line.starts_with("OFFER:"))
        .context("No OFFER: found in the data")?;

    let offer_json = offer_line.trim_start_matches("OFFER:");
    let offer = serde_json::from_str::<RTCSessionDescription>(offer_json)
        .context("Failed to parse offer")?;

    println!("Successfully parsed offer");
    Ok(offer)
}

// Parse SDP answer from input text
pub fn parse_answer(response: &str) -> Result<RTCSessionDescription> {
    let answer_line = response
        .lines()
        .find(|line| line.starts_with("ANSWER:"))
        .context("No ANSWER: found in the response")?;

    let answer_json = answer_line.trim_start_matches("ANSWER:");
    let answer = serde_json::from_str::<RTCSessionDescription>(answer_json)
        .context("Failed to parse answer")?;

    println!("Successfully parsed answer");
    Ok(answer)
}

// Process ICE candidates from SDP data
pub async fn process_ice_candidates(data: &str, pc: &Arc<RTCPeerConnection>) -> Result<()> {
    // Look for ICE candidates section and parse each line as a candidate
    let mut in_candidates_section = false;
    let mut candidates_json = String::new();

    for line in data.lines() {
        if line.contains("ICE_CANDIDATES:") {
            in_candidates_section = true;
            continue;
        }

        if in_candidates_section && !line.contains("END OF SECTION") {
            candidates_json.push_str(line.trim());
        } else if line.contains("END OF SECTION") {
            break;
        }
    }

    if !candidates_json.is_empty() {
        match serde_json::from_str::<Vec<String>>(&candidates_json) {
            Ok(remote_candidates) => {
                println!(
                    "Successfully parsed {} ICE candidates",
                    remote_candidates.len()
                );

                let mut success_count = 0;
                let mut error_count = 0;

                // Using &remote_candidates to avoid moving the vector
                for candidate_str in &remote_candidates {
                    // Use the conversion function for all candidates
                    match convert_to_ice_candidate(candidate_str).await {
                        Ok(candidate) => match pc.add_ice_candidate(candidate).await {
                            Ok(_) => {
                                success_count += 1;
                            }
                            Err(e) => {
                                debug!("Warning: Failed to add ICE candidate: {}", e);
                                error_count += 1;
                            }
                        },
                        Err(e) => {
                            debug!("Warning: Failed to convert ICE candidate: {}", e);
                            error_count += 1;
                        }
                    }
                }

                println!(
                    "Added {}/{} ICE candidates successfully",
                    success_count,
                    remote_candidates.len()
                );
                if error_count > 0 {
                    println!(
                        "({} candidates failed but connection may still work)",
                        error_count
                    );
                }
            }
            Err(e) => {
                warn!("Error parsing ICE candidates: {}", e);
                println!("Failed to parse ICE candidates, but will continue without them");
                println!("The connection may still work but might be less reliable");
            }
        }
    } else {
        println!("No ICE candidates found in the data");
    }

    Ok(())
}

// Convert JSON string to ICE candidate
async fn convert_to_ice_candidate(candidate_str: &str) -> Result<RTCIceCandidateInit> {
    // Parse the raw JSON string to get the fields
    let raw_json: serde_json::Value =
        serde_json::from_str(candidate_str).context("Failed to parse ICE candidate JSON")?;

    // Extract the needed fields with better error handling
    let foundation = raw_json["foundation"].as_str().unwrap_or("").to_string();
    let component = raw_json["component"].as_u64().unwrap_or(1) as u16;
    let protocol = raw_json["protocol"].as_str().unwrap_or("udp").to_string();
    let priority = raw_json["priority"].as_u64().unwrap_or(0) as u32;
    let address = raw_json["address"].as_str().unwrap_or("").to_string();
    let port = raw_json["port"].as_u64().unwrap_or(0) as u16;
    let typ = raw_json["typ"].as_str().unwrap_or("").to_string();

    // Additional fields that might be present
    let tcp_type = if let Some(tcp_type) = raw_json["tcptype"].as_str() {
        format!(" tcptype {}", tcp_type)
    } else {
        String::new()
    };

    // Create the candidate string in standard format
    let candidate_str = format!(
        "candidate:{} {} {} {} {} {} typ {}{}",
        foundation, component, protocol, priority, address, port, typ, tcp_type
    );

    // Create the RTCIceCandidateInit structure
    let candidate = RTCIceCandidateInit {
        candidate: candidate_str,
        ..Default::default()
    };

    Ok(candidate)
}
