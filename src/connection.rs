use anyhow::Result;
use log::{error, info, warn};
use std::io::{self, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_init::RTCDataChannelInit;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::policy::ice_transport_policy::RTCIceTransportPolicy;
use webrtc::peer_connection::RTCPeerConnection;

// Create and configure a new peer connection
pub async fn create_peer_connection(use_turn: bool) -> Result<Arc<RTCPeerConnection>> {
    // Create a MediaEngine object to configure the supported codec
    let mut m = MediaEngine::default();

    // Create an InterceptorRegistry
    let registry = interceptor::registry::Registry::new();

    // Register default interceptors
    let registry = register_default_interceptors(registry, &mut m)?;

    // Enhanced ICE server configuration with fallbacks
    let mut ice_servers = vec![RTCIceServer {
        urls: vec![
            "stun:stun.l.google.com:19302".to_owned(),
            "stun:stun1.l.google.com:19302".to_owned(),
            "stun:stun2.l.google.com:19302".to_owned(),
            "stun:stun3.l.google.com:19302".to_owned(),
            "stun:stun4.l.google.com:19302".to_owned(),
        ],
        ..Default::default()
    }];

    // Add TURN servers for situations where direct connections aren't possible
    if use_turn {
        ice_servers.push(RTCIceServer {
            urls: vec![
                // These would need to be replaced with actual TURN servers
                // "turn:turn.example.org:3478".to_owned(),
            ],
            username: "".to_owned(), // Credentials would be needed
            credential: "".to_owned(),
            ..Default::default()
        });
    }

    // Create the API object with more extensive configuration
    let api = APIBuilder::new()
        .with_interceptor_registry(registry)
        .build();

    // Create a new RTCPeerConnection with enhanced configuration
    let peer_connection = Arc::new(
        api.new_peer_connection(RTCConfiguration {
            ice_servers,
            ice_transport_policy: RTCIceTransportPolicy::All,
            ..Default::default()
        })
        .await?,
    );

    Ok(peer_connection)
}

// Setup a data channel for messaging
pub async fn setup_data_channel(
    pc: Arc<RTCPeerConnection>,
    channel_name: &str,
    is_offerer: bool,
) -> Result<Arc<Mutex<Option<Arc<RTCDataChannel>>>>> {
    let data_channel = Arc::new(Mutex::new(None as Option<Arc<RTCDataChannel>>));
    let data_channel_clone = Arc::clone(&data_channel);

    if is_offerer {
        // Create a datachannel with enhanced settings for better reliability
        let dc = pc
            .create_data_channel(
                channel_name,
                Some(RTCDataChannelInit {
                    ordered: Some(true),
                    max_retransmits: Some(3), // Add retry logic
                    ..Default::default()
                }),
            )
            .await?;

        info!("Created data channel: {}", channel_name);

        let d1 = Arc::clone(&dc);
        let data_channel_clone3 = Arc::clone(&data_channel_clone);
        dc.on_open(Box::new(move || {
            info!("Data channel '{}' opened", d1.label());

            // Update the data channel in the shared state when it opens
            let d1_clone = Arc::clone(&d1);
            let data_channel_clone4 = Arc::clone(&data_channel_clone3);
            tokio::spawn(async move {
                *data_channel_clone4.lock().await = Some(d1_clone);
                info!("Data channel state updated");
            });

            Box::pin(async {})
        }));

        // Improved message handling with proper error handling
        let _d2 = Arc::clone(&dc);
        dc.on_message(Box::new(move |msg| {
            match String::from_utf8(msg.data.to_vec()) {
                Ok(message) => {
                    // Enhanced message display with timestamp and formatting
                    let now = chrono::Local::now().format("%H:%M:%S");
                    println!("[{}] Received: {}", now, message);
                }
                Err(e) => {
                    warn!("Received invalid UTF-8 data: {}", e);
                    println!("Received message with invalid encoding");
                }
            }
            Box::pin(async {})
        }));

        // Add error handler for data channel
        dc.on_error(Box::new(move |err| {
            error!("Data channel error: {}", err);
            Box::pin(async {})
        }));

        *data_channel.lock().await = Some(dc);
    } else {
        // Register data channel creation handling
        pc.on_data_channel(Box::new(move |dc| {
            info!("New data channel: {}", dc.label());

            let d1 = Arc::clone(&dc);
            let data_channel_clone3 = Arc::clone(&data_channel_clone);
            dc.on_open(Box::new(move || {
                info!("Data channel '{}' opened", d1.label());

                // Update the data channel in the shared state when it opens
                let d1_clone = Arc::clone(&d1);
                let data_channel_clone4 = Arc::clone(&data_channel_clone3);
                tokio::spawn(async move {
                    *data_channel_clone4.lock().await = Some(d1_clone);
                    info!("Data channel state updated");
                });

                Box::pin(async {})
            }));

            // Improved message handling with proper error handling
            let _d2 = Arc::clone(&dc);
            dc.on_message(Box::new(move |msg| {
                match String::from_utf8(msg.data.to_vec()) {
                    Ok(message) => {
                        // Enhanced message display with timestamp and formatting
                        let now = chrono::Local::now().format("%H:%M:%S");
                        println!("[{}] Received: {}", now, message);
                    }
                    Err(e) => {
                        warn!("Received invalid UTF-8 data: {}", e);
                        println!("Received message with invalid encoding");
                    }
                }
                Box::pin(async {})
            }));

            // Add error handler for data channel
            dc.on_error(Box::new(move |err| {
                error!("Data channel error: {}", err);
                Box::pin(async {})
            }));

            let dc_clone = Arc::clone(&dc);
            let data_channel_clone2 = Arc::clone(&data_channel_clone);
            tokio::spawn(async move {
                *data_channel_clone2.lock().await = Some(dc_clone);
            });
            Box::pin(async {})
        }));
    }

    // Set the handler for Peer connection state with more detailed reporting
    pc.on_peer_connection_state_change(Box::new(move |s| {
        info!("Peer Connection State has changed: {}", s);
        match s {
            RTCPeerConnectionState::Failed => {
                error!("Peer connection failed. This may be due to network issues or incompatible configurations.");
            }
            RTCPeerConnectionState::Disconnected => {
                warn!("Peer connection temporarily disconnected. Will attempt to recover...");
            }
            RTCPeerConnectionState::Closed => {
                info!("Peer connection closed. Exiting chat.");
            }
            RTCPeerConnectionState::Connected => {
                info!("Peer connection fully established!");
            }
            _ => {}
        }
        Box::pin(async {})
    }));

    Ok(data_channel)
}

// Monitor connection state with timeout
pub async fn monitor_connection_state(
    pc: Arc<RTCPeerConnection>,
    timeout: Duration,
    start_time: std::time::Instant,
) -> Result<bool> {
    let connection_established = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let connection_established_clone = Arc::clone(&connection_established);

    // Spawn a task to monitor the connection state
    let monitor_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(500));
        let mut dots = 0;

        print!("Establishing connection");
        io::stdout().flush().unwrap();

        loop {
            interval.tick().await;
            let state = pc.connection_state();

            // Progress indicator
            print!(".");
            dots = (dots + 1) % 3;
            for _ in 0..3 - dots {
                print!(" ");
            }
            print!("\r");
            print!("Establishing connection");
            for _ in 0..dots + 1 {
                print!(".");
            }
            io::stdout().flush().unwrap();

            if state == RTCPeerConnectionState::Connected {
                println!("\nConnection established successfully!      ");
                connection_established_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                break;
            } else if state == RTCPeerConnectionState::Failed
                || state == RTCPeerConnectionState::Closed
            {
                println!("\nConnection failed or closed              ");
                break;
            }

            // Check for timeout
            if start_time.elapsed() > timeout {
                println!("\nConnection attempt timed out             ");
                break;
            }
        }
    });

    // Handle the JoinError properly
    match monitor_handle.await {
        Ok(_) => {
            // Return whether the connection was successfully established
            Ok(connection_established.load(std::sync::atomic::Ordering::SeqCst))
        }
        Err(e) => {
            error!("Error in connection monitoring task: {}", e);
            Ok(false)
        }
    }
}
