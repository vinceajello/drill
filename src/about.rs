/// Show the about dialog using native system dialog
pub fn show_about_window() {
    use crate::logs::log_print;
    
    log_print("Opening About dialog...");
    
    std::thread::spawn(|| {
        let message = format!(
            "Drill\n\
            Version 0.1.0\n\n\
            A multi-platform tunnel drilling application\n\
            for macOS, Windows, and Linux\n\n\
            Platform: {}\n\n\
            © 2025",
            crate::get_platform()
        );
        
        rfd::MessageDialog::new()
            .set_title("About Drill")
            .set_description(&message)
            .set_level(rfd::MessageLevel::Info)
            .show();
            
        log_print("About dialog closed");
    });
}
