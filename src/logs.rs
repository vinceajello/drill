use std::fs::File;
use std::io::Write;

pub struct Logger {
    log_file: File,
}

impl Logger {
    pub fn new(log_file: File) -> Self {
        Logger { log_file }
    }

    pub fn log_print(&mut self, message: &str) {
        // Print to console
        println!("{}", message);
        // Write to log file
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        let log_line = format!("[{}] {}\n", timestamp, message);
        let _ = self.log_file.write_all(log_line.as_bytes());
    }
}
