use anyhow::Result;
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use webrtc::data_channel::data_channel_state::RTCDataChannelState;
use webrtc::data_channel::RTCDataChannel;

// Enhanced message loop with more features
pub async fn enhanced_message_loop(dc: Arc<Mutex<Option<Arc<RTCDataChannel>>>>) -> Result<()> {
    let stdin = io::stdin();
    let mut message_history = Vec::new();
    let mut _history_position = 0;

    println!("Waiting for data channel to be ready...");
    let mut is_ready = false;
    let start_time = std::time::Instant::now();
    let timeout = Duration::from_secs(30); // 30 seconds timeout
    let mut progress_counter = 0;

    while !is_ready {
        {
            let dc_lock = dc.lock().await;
            if let Some(ref data_channel) = *dc_lock {
                // Check if data channel is open
                if data_channel.ready_state() == RTCDataChannelState::Open {
                    is_ready = true;
                    println!("\rData channel is now open and ready!            ");
                }
            }
        }

        if !is_ready {
            if start_time.elapsed() > timeout {
                println!("\rTimeout waiting for data channel to open          ");
                println!("Proceeding anyway - the channel may open later");
                break;
            }

            // Show progress indicator
            print!("\rWaiting for connection");
            for _ in 0..progress_counter {
                print!(".");
            }
            print!("   ");
            io::stdout().flush()?;
            progress_counter = (progress_counter + 1) % 4;

            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    println!("\n===== CHAT SESSION STARTED =====");
    println!("Enter messages (or type '/exit' to quit, '/help' for commands):");

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        stdin.lock().read_line(&mut input)?;
        let input = input.trim().to_string();

        // Command handling
        if input.starts_with('/') {
            match input.as_str() {
                "/exit" | "/quit" => {
                    println!("Exiting chat...");
                    break;
                }
                "/help" => {
                    println!("Available commands:");
                    println!("  /exit, /quit - Exit the chat");
                    println!("  /help       - Show this help message");
                    println!("  /status     - Show connection status");
                    println!("  /clear      - Clear the screen");
                    println!("  /history    - Show message history");
                    continue;
                }
                "/status" => {
                    let dc_lock = dc.lock().await;
                    if let Some(ref data_channel) = *dc_lock {
                        println!("Data channel state: {:?}", data_channel.ready_state());
                    } else {
                        println!("Data channel not yet created");
                    }
                    continue;
                }
                "/clear" => {
                    // Clear screen with ANSI escape codes (might not work on all terminals)
                    print!("\x1B[2J\x1B[1;1H");
                    continue;
                }
                "/history" => {
                    if message_history.is_empty() {
                        println!("No message history yet");
                    } else {
                        println!("Message history:");
                        for (i, msg) in message_history.iter().enumerate() {
                            println!("  {}. {}", i + 1, msg);
                        }
                    }
                    continue;
                }
                _ => {
                    println!(
                        "Unknown command: {}. Type /help for available commands",
                        input
                    );
                    continue;
                }
            }
        }

        // Skip empty messages
        if input.is_empty() {
            continue;
        }

        // Add to history
        message_history.push(input.clone());
        _history_position = message_history.len();

        let dc_lock = dc.lock().await;
        if let Some(ref data_channel) = *dc_lock {
            if data_channel.ready_state() == RTCDataChannelState::Open {
                // Add timestamp to messages
                let timestamp = chrono::Local::now().format("%H:%M:%S");
                let formatted_message = format!("[{}] {}", timestamp, input);

                match data_channel.send_text(formatted_message).await {
                    Ok(_) => {
                        // Print confirmation
                        println!("(Message sent)");
                    }
                    Err(e) => {
                        println!("Error sending message: {}", e);
                    }
                }
            } else {
                println!(
                    "Data channel not open, current state: {:?}",
                    data_channel.ready_state()
                );

                // Attempt recovery
                println!("Waiting for channel to open...");

                // Drop the lock before sleeping
                drop(dc_lock);

                // Wait a moment to see if the channel opens
                tokio::time::sleep(Duration::from_secs(2)).await;

                // Try sending again
                let dc_lock = dc.lock().await;
                if let Some(ref data_channel) = *dc_lock {
                    if data_channel.ready_state() == RTCDataChannelState::Open {
                        let timestamp = chrono::Local::now().format("%H:%M:%S");
                        let formatted_message = format!("[{}] {}", timestamp, input);

                        match data_channel.send_text(formatted_message).await {
                            Ok(_) => println!("Message sent on retry"),
                            Err(e) => println!("Error sending message on retry: {}", e),
                        }
                    } else {
                        println!(
                            "Data channel still not open, current state: {:?}",
                            data_channel.ready_state()
                        );
                        println!("Try again in a few moments");
                    }
                }
            }
        } else {
            println!("Data channel not yet created, message queued for delivery");
        }
    }

    Ok(())
}
