// FIFO Writer Implementation
// Based on spi-lib pattern: open-write-close with O_NONBLOCK

use std::ffi::CString;
use std::time::Instant;
use libc::{O_WRONLY, O_NONBLOCK};
use log::debug;

pub struct FifoWriter {
    path: CString,
    interval_secs: f32,
    last_write: Option<Instant>,
}

impl FifoWriter {
    pub fn new(path: &str, interval_secs: f32) -> Result<Self, std::io::Error> {
        let path_cstring = CString::new(path)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
        
        // Create FIFO if it doesn't exist (same as spi-lib)
        unsafe {
            libc::mkfifo(path_cstring.as_ptr(), 0o666);
        }
        
        debug!("FIFO created/verified at: {} (interval: {:.1}s)", path, interval_secs);
        
        Ok(Self {
            path: path_cstring,
            interval_secs,
            last_write: None,
        })
    }
    
    /// Write data using spi-lib pattern: open-write-close with O_NONBLOCK
    pub fn write_data(&self, data: &str) -> Result<(), std::io::Error> {
        unsafe {
            // CRITICAL: Same pattern as spi-lib - O_WRONLY | O_NONBLOCK
            let fd = libc::open(self.path.as_ptr(), O_WRONLY | O_NONBLOCK);
            
            if fd >= 0 {
                // Reader is connected, write the data
                let data_bytes = data.as_bytes();
                let written = libc::write(
                    fd, 
                    data_bytes.as_ptr() as *const libc::c_void, 
                    data_bytes.len()
                );
                libc::close(fd);
                
                if written < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                
                debug!("FIFO write successful: {} bytes", written);
                Ok(())
            } else {
                // No reader connected - this is normal, don't treat as error
                // (spi-lib silently continues in this case)
                debug!("FIFO write skipped: no reader connected");
                Ok(())
            }
        }
    }
    
    /// Write JSON data (enhanced format)
    pub fn write_json(&self, json_data: &serde_json::Value) -> Result<(), std::io::Error> {
        let json_string = format!("{}\n", json_data.to_string());
        self.write_data(&json_string)
    }
    
    /// Write simple format for BGT60TR13C compatibility
    pub fn write_simple(&self, presence_state: i32, distance: f32) -> Result<(), std::io::Error> {
        let simple_data = format!("{} {:.2}\n", presence_state, distance);
        self.write_data(&simple_data)
    }
    
    /// Write status messages (startup/shutdown)
    pub fn write_status(&self, status: &str) -> Result<(), std::io::Error> {
        let status_data = format!("STATUS {}\n", status);
        self.write_data(&status_data)
    }
    
    /// Check if it's time to write to FIFO (spi-lib compatible timing)
    pub fn should_write(&mut self) -> bool {
        // If interval is 0, write every time (immediate mode)
        if self.interval_secs <= 0.0 {
            return true;
        }
        
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
                    debug!("FIFO write interval reached: {:.1}s elapsed", elapsed);
                    true
                } else {
                    false
                }
            }
        }
    }
    
    /// Write data with timing control (spi-lib pattern)
    pub fn write_timed_json(&mut self, json_data: &serde_json::Value) -> Result<bool, std::io::Error> {
        if self.should_write() {
            self.write_json(json_data)?;
            Ok(true) // Data was written
        } else {
            Ok(false) // Skipped due to timing
        }
    }
    
    /// Write data with timing control (spi-lib pattern)
    pub fn write_timed_simple(&mut self, presence_state: i32, distance: f32) -> Result<bool, std::io::Error> {
        if self.should_write() {
            self.write_simple(presence_state, distance)?;
            Ok(true) // Data was written
        } else {
            Ok(false) // Skipped due to timing
        }
    }
}

#[derive(Debug, Clone)]
pub enum FifoFormat {
    Simple,  // BGT60TR13C compatibility: "1 2.45"
    Json,    // Enhanced XM125 format
}

impl std::str::FromStr for FifoFormat {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "simple" => Ok(FifoFormat::Simple),
            "json" => Ok(FifoFormat::Json),
            _ => Err(format!("Invalid FIFO format: {}. Use 'simple' or 'json'", s)),
        }
    }
}
