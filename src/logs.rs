use std::fs;
use std::io::Write;
use std::sync::Mutex;

static LOG_FILE: Mutex<Option<fs::File>> = Mutex::new(None);

/// Initialize the log file
pub fn init_log_file(log_file: fs::File) {
    *LOG_FILE.lock().unwrap() = Some(log_file);
}

/// Print a message to console and write it to the log file
pub fn log_print(message: &str) {
    // Print to console
    println!("{}", message);
    
    // Write to log file
    if let Ok(mut log_file_guard) = LOG_FILE.lock() {
        if let Some(ref mut log_file) = *log_file_guard {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let log_line = format!("[{}] {}\n", timestamp, message);
            let _ = log_file.write_all(log_line.as_bytes());
        }
    }
}
