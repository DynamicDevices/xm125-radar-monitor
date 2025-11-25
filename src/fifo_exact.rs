// Exact spi-lib behavior implementation
// This matches the BGT60TR13C timing pattern exactly

use crate::fifo::FifoWriter;
use crate::radar::XM125Radar;
use crate::error::RadarError;
use tokio::time::{sleep, Duration, Instant};
use log::info;

pub struct FifoController {
    writer: FifoWriter,
    interval_secs: f32,
    last_write: Option<Instant>,
}

impl FifoController {
    pub fn new(fifo_path: &str, interval_secs: f32) -> Result<Self, std::io::Error> {
        let writer = FifoWriter::new(fifo_path)?;
        
        // Send startup status (same as spi-lib)
        let _ = writer.write_status("Starting up");
        
        Ok(Self {
            writer,
            interval_secs,
            last_write: None,
        })
    }
    
    /// Check if it's time to write to FIFO (matches spi-lib timing)
    pub fn should_write_fifo(&mut self) -> bool {
        let now = Instant::now();
        
        match self.last_write {
            None => {
                // First write
                self.last_write = Some(now);
                true
            }
            Some(last) => {
                let elapsed = now.duration_since(last).as_secs_f32();
                if elapsed >= self.interval_secs {
                    self.last_write = Some(now);
                    true
                } else {
                    false
                }
            }
        }
    }
    
    /// Write presence data (only when should_write_fifo() returns true)
    pub fn write_presence_data(&self, presence_state: i32, distance: f32) -> Result<(), std::io::Error> {
        // Exact same format as spi-lib: "%d %f\n"
        self.writer.write_simple(presence_state, distance)
    }
    
    /// Send exit status
    pub fn shutdown(&self) -> Result<(), std::io::Error> {
        self.writer.write_status("App exit")
    }
}

/// Run in exact spi-lib mode: continuous measurement, timed FIFO output
pub async fn run_spi_lib_compatible_mode(
    radar: &mut XM125Radar,
    fifo_path: &str,
    interval_secs: f32,
) -> Result<(), RadarError> {
    let mut fifo_controller = FifoController::new(fifo_path, interval_secs)
        .map_err(|e| RadarError::DeviceError {
            message: format!("Failed to initialize FIFO: {}", e),
        })?;
    
    info!("Starting spi-lib compatible mode: FIFO updates every {:.1}s", interval_secs);
    
    // Continuous measurement loop (like spi-lib)
    loop {
        // Measure presence (this happens continuously, like spi-lib processes frames)
        match radar.measure_presence().await {
            Ok(result) => {
                // Only write to FIFO if enough time has elapsed (spi-lib pattern)
                if fifo_controller.should_write_fifo() {
                    let presence_state = if result.presence_detected { 1 } else { 0 };
                    let _ = fifo_controller.write_presence_data(presence_state, result.presence_distance);
                    info!("FIFO update: {} {:.2}", presence_state, result.presence_distance);
                }
                // Continue measuring without delay (like spi-lib continuous processing)
            }
            Err(e) => {
                eprintln!("Measurement failed: {}", e);
                sleep(Duration::from_millis(100)).await; // Brief pause on error
            }
        }
    }
}
