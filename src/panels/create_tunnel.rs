use crate::logs::log_print;
use crate::tunnels::Tunnel;

/// Show the create tunnel dialog and return a new tunnel if created
pub fn show_create_tunnel_dialog() -> Option<Tunnel> {
    log_print("Opening Create Tunnel dialog...");
    
    // Launch the create dialog as a separate process
    // This avoids the main thread requirement on macOS
    let exe_path = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    
    let create_exe = exe_path.join("drill-create");
    
    match std::process::Command::new(&create_exe).output() {
        Ok(output) => {
            log_print("Create tunnel dialog closed");
            
            // Parse output for tunnel data
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stdout.lines().find(|l| l.starts_with("TUNNEL_CREATED:")) {
                log_print(&format!("Tunnel data received: {}", line));
                // TODO: Parse tunnel data and return Tunnel object
            }
            
            None
        }
        Err(e) => {
            log_print(&format!("Error launching create dialog: {}", e));
            log_print(&format!("Tried to run: {:?}", create_exe));
            None
        }
    }
}
