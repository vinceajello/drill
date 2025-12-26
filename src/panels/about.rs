// use crate::logs::log_print;

/// Show the about dialog using iced
pub fn show_about_window() {
    // log_print("Opening About dialog...");
    
    // Launch the about dialog as a separate process
    // This avoids the main thread requirement on macOS
    std::thread::spawn(|| {
        let exe_path = std::env::current_exe()
            .ok()
            .and_then(|path| path.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::path::PathBuf::from("."));
        
        let about_exe = exe_path.join("drill-about");
        
        match std::process::Command::new(&about_exe).spawn() {
            Ok(mut child) => {
                let _ = child.wait();
                // log_print("About dialog closed");
            }
            Err(e) => {
                // log_print(&format!("Error launching about dialog: {}", e));
                // log_print(&format!("Tried to run: {:?}", about_exe));
            }
        }
    });
}
