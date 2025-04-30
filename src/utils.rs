use std::io::{self, Write};
use std::time::Duration;

// Print a progress indicator with dots that animate
pub fn print_progress(message: &str, progress_counter: usize) -> io::Result<()> {
    print!("\r{}", message);
    for _ in 0..progress_counter {
        print!(".");
    }
    print!("   ");
    io::stdout().flush()?;
    Ok(())
}

// Wait for a specified time with a spinner animation
pub async fn animated_wait(message: &str, duration: Duration) -> io::Result<()> {
    let steps = (duration.as_millis() / 100) as usize;
    let spinner = vec!["|", "/", "-", "\\"];

    for i in 0..steps {
        print!("\r{} {} ", message, spinner[i % spinner.len()]);
        io::stdout().flush()?;
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    println!();
    Ok(())
}

// Format bytes in human-readable form
pub fn format_bytes(bytes: usize) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let bytes = bytes as f64;
    let i = (bytes.ln() / 1024_f64.ln()).floor() as usize;

    if i >= UNITS.len() {
        return format!("{:.2} {}", bytes / 1024_f64.powi(4), UNITS[4]);
    }

    format!("{:.2} {}", bytes / 1024_f64.powi(i as i32), UNITS[i])
}

// Add timestamp to message
pub fn add_timestamp(message: &str) -> String {
    let timestamp = chrono::Local::now().format("%H:%M:%S");
    format!("[{}] {}", timestamp, message)
}

// Convert duration to human-readable string
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();

    if total_secs < 60 {
        return format!("{}s", total_secs);
    }

    let mins = total_secs / 60;
    let secs = total_secs % 60;

    if mins < 60 {
        return format!("{}m {}s", mins, secs);
    }

    let hours = mins / 60;
    let mins = mins % 60;

    format!("{}h {}m {}s", hours, mins, secs)
}
