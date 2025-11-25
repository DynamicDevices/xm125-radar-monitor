// Example of dedicated FIFO mode implementation
// Add this to Commands enum in cli.rs:

Commands::FifoMode {
    /// FIFO output interval in seconds (like spi-lib's 5 second interval)
    #[arg(long, default_value = "5.0", help = "FIFO output interval in seconds")]
    interval: f32,
    
    /// FIFO path
    #[arg(long, default_value = "/tmp/presence", help = "FIFO output path")]
    fifo_path: String,
    
    /// Output format
    #[arg(long, default_value = "simple", help = "FIFO output format: simple or json")]
    format: fifo::FifoFormat,
    
    /// Presence range configuration
    #[arg(long, help = "Presence range preset")]
    range: Option<PresenceRange>,
    
    /// Custom minimum range
    #[arg(long, help = "Custom minimum detection range in meters")]
    min_range: Option<f32>,
    
    /// Custom maximum range  
    #[arg(long, help = "Custom maximum detection range in meters")]
    max_range: Option<f32>,
},

// Then add to execute_command:
Commands::FifoMode { interval, fifo_path, format, range, min_range, max_range } => {
    // Configure radar for presence detection
    radar.set_detector_mode(radar::DetectorMode::Presence);
    configure_presence_parameters(radar, range.as_ref(), *min_range, *max_range, None, None, &ProfileMode::Auto)?;
    
    // Create FIFO writer
    let fifo_writer = FifoWriter::new(fifo_path)?;
    fifo_writer.write_status("Starting up")?;
    
    // Run continuous monitoring with timed FIFO output (like spi-lib)
    run_fifo_mode(radar, &fifo_writer, *interval, format).await?;
    
    fifo_writer.write_status("App exit")?;
}

async fn run_fifo_mode(
    radar: &mut XM125Radar,
    fifo_writer: &FifoWriter,
    interval_secs: f32,
    format: &fifo::FifoFormat,
) -> Result<(), RadarError> {
    use tokio::time::{sleep, Duration};
    
    let interval_duration = Duration::from_secs_f32(interval_secs);
    
    loop {
        let result = radar.measure_presence().await?;
        
        match format {
            fifo::FifoFormat::Simple => {
                let presence_state = if result.presence_detected { 1 } else { 0 };
                fifo_writer.write_simple(presence_state, result.presence_distance)?;
            }
            fifo::FifoFormat::Json => {
                let json_data = serde_json::json!({
                    "timestamp": chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
                    "presence_detected": result.presence_detected,
                    "presence_distance_m": result.presence_distance,
                    "intra_score": result.intra_presence_score,
                    "inter_score": result.inter_presence_score
                });
                fifo_writer.write_json(&json_data)?;
            }
        }
        
        sleep(interval_duration).await;
    }
}
