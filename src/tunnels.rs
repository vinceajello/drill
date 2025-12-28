use std::fs;
use std::path::PathBuf;
use std::io::Write;
use std::process::{Command, Child, Stdio};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;
use crate::error::{DrillResult, DrillError};



/// Enhanced tunnel status with error details
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TunnelStatus {
    Disconnected,
    Connecting,
    Connected { 
        connected_at: std::time::SystemTime,
    },
    Error { 
        error: String,
        occurred_at: std::time::SystemTime,
    },
    #[allow(dead_code)]
    Reconnecting {
        attempt: u32,
    },
}



/// Status update events from monitoring tasks
#[derive(Debug, Clone)]
pub enum StatusUpdate {
    Connecting(String),
    Connected(String),
    Error(String, String),
    Disconnected(String),
}

/// Information about an active tunnel process
struct ActiveTunnel {
    process: Child,
    #[allow(dead_code)]
    started_at: Instant,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Tunnel {
    pub id: String,
    pub name: String,
    pub local_host: String,
    pub local_port: String,
    pub remote_host: String,
    pub remote_port: String,
    pub ssh_user: String,
    pub ssh_host: String,
    pub ssh_port: String,
    #[serde(default)]
    pub private_key: String,
}

pub struct TunnelManager {
    tunnels: Vec<Tunnel>,
    active_processes: HashMap<String, ActiveTunnel>,
    tunnel_status: HashMap<String, TunnelStatus>,
    status_tx: Option<broadcast::Sender<StatusUpdate>>,
}

impl TunnelManager {
    pub fn new() -> Self {
        TunnelManager {
            tunnels: Vec::new(),
            active_processes: HashMap::new(),
            tunnel_status: HashMap::new(),
            status_tx: None,
        }
    }
    
    /// Set the status update channel
    pub fn set_status_channel(&mut self, tx: broadcast::Sender<StatusUpdate>) {
        self.status_tx = Some(tx);
    }

    /// Send a status update
    fn send_status_update(&self, update: StatusUpdate) {
        if let Some(tx) = &self.status_tx {
            let _ = tx.send(update);
        }
    }

    /// Load tunnels from the tunnels file
    pub fn load_tunnels(tunnels_file: &PathBuf) -> DrillResult<Vec<Tunnel>> {
        if !tunnels_file.exists() {
            // logger.log_print("Tunnels file not found, returning empty list");
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(tunnels_file)?;
        let tunnels: Vec<Tunnel> = serde_yaml::from_str(&content)?;
        // logger.log_print(&format!("Loaded {} tunnel(s)", tunnels.len()));
        Ok(tunnels)
    }

    /// Save tunnels to the tunnels file
    pub fn save_tunnels(tunnels_file: &PathBuf, tunnels: &Vec<Tunnel>) -> DrillResult<()> {
        let yaml = serde_yaml::to_string(tunnels)?;
        let mut file = fs::File::create(tunnels_file)?;
        file.write_all(yaml.as_bytes())?;
        // logger.log_print(&format!("Saved {} tunnel(s)", tunnels.len()));
        Ok(())
    }

    /// Set the tunnels for this manager
    pub fn set_tunnels(&mut self, tunnels: Vec<Tunnel>) {
        self.tunnels = tunnels;
    }

    /// Get all tunnels
    pub fn get_tunnels(&self) -> &Vec<Tunnel> {
        &self.tunnels
    }

    /// Add a new tunnel
    pub fn add_tunnel(&mut self, tunnel: Tunnel) {
        self.tunnels.push(tunnel);
    }

    /// Update an existing tunnel by ID
    pub fn update_tunnel(&mut self, tunnel_id: &str, updated_tunnel: Tunnel) -> DrillResult<()> {
        // Find tunnel by ID
        if let Some(index) = self.tunnels.iter().position(|t| t.id == tunnel_id) {
            // If tunnel is active, we may need to restart it with new settings
            let _old_name = self.tunnels[index].name.clone();
            
            // Update the tunnel
            self.tunnels[index] = updated_tunnel;
            
            // logger.log_print(&format!("Tunnel with ID '{}' updated", tunnel_id));
            Ok(())
        } else {
            Err(DrillError::Tunnel(format!("Tunnel with ID '{}' not found", tunnel_id)))
        }
    }

    /// Check if a tunnel is active
    pub fn is_tunnel_active(&self, tunnel_name: &str) -> bool {
        self.active_processes.contains_key(tunnel_name)
    }

    /// Get the status of a tunnel
    pub fn get_tunnel_status(&self, tunnel_name: &str) -> TunnelStatus {
        self.tunnel_status.get(tunnel_name).cloned().unwrap_or(TunnelStatus::Disconnected)
    }

    /// Start a tunnel with comprehensive error monitoring
    pub fn start_tunnel(&mut self, tunnel: &Tunnel) -> DrillResult<()> {
        if self.active_processes.contains_key(&tunnel.name) {
            // logger.log_print(&format!("Tunnel '{}' is already active", tunnel.name));
            return Ok(());
        }

        // Set status to connecting
        self.tunnel_status.insert(tunnel.name.clone(), TunnelStatus::Connecting);
        // Send status update
        self.send_status_update(StatusUpdate::Connecting(tunnel.name.clone()));

        // Build SSH command with enhanced error detection
        let local_forward = format!(
            "{}:{}:{}",
            tunnel.local_port, tunnel.remote_host, tunnel.remote_port
        );
        let remote = format!("{}@{}", tunnel.ssh_user, tunnel.ssh_host);

        // logger.log_print(&format!(
        //     "Starting tunnel '{}': ssh -L {} -N -p {} {}",
        //     tunnel.name, local_forward, tunnel.ssh_port, remote
        // ));

        let mut command = Command::new("ssh");
        // Add private key if provided
        if !tunnel.private_key.trim().is_empty() {
            command.arg("-i").arg(&tunnel.private_key);
        }
        command
            .arg("-L")
            .arg(&local_forward)
            .arg("-N") // Don't execute remote command
            .arg("-v") // Verbose mode for better error messages
            .arg("-o")
            .arg("ServerAliveInterval=60")
            .arg("-o")
            .arg("ServerAliveCountMax=3")
            .arg("-o")
            .arg("ExitOnForwardFailure=yes") // Exit if port forwarding fails
            .arg("-o")
            .arg("ConnectTimeout=10") // 10 second connection timeout
            .arg("-p")
            .arg(&tunnel.ssh_port)
            .arg(&remote)
            .stderr(Stdio::piped()) // Capture stderr for error detection
            .stdout(Stdio::null())
            .stdin(Stdio::null());

        // On Windows, suppress terminal window
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        match command.spawn() {
            Ok(mut child) => {
                let tunnel_name = tunnel.name.clone();
                let _process_id = child.id();
                let _stderr = child.stderr.take();
                let active_tunnel = ActiveTunnel {
                    process: child,
                    started_at: Instant::now(),
                };
                self.active_processes.insert(tunnel_name.clone(), active_tunnel);

                // Initial connection verification (give it a moment to start)
                std::thread::sleep(Duration::from_millis(500));

                // Check if process is still running
                if let Some(active) = self.active_processes.get_mut(&tunnel_name) {
                    match active.process.try_wait() {
                        Ok(Some(status)) => {
                            // Process already exited
                            self.active_processes.remove(&tunnel_name);
                            let error = DrillError::SshProcess(format!("Process exited immediately with status: {}", status));
                            self.tunnel_status.insert(
                                tunnel_name.clone(),
                                TunnelStatus::Error {
                                    error: error.to_string(),
                                    occurred_at: std::time::SystemTime::now(),
                                }
                            );
                            self.send_status_update(StatusUpdate::Error(tunnel_name.clone(), DrillError::SshProcess(error.to_string()).to_string()));
                            // logger.log_print(&format!("Error starting tunnel '{}': {}", tunnel_name, error));
                            return Err(error);
                        }
                        Ok(None) => {
                            // Process is running - mark as connected
                            self.tunnel_status.insert(
                                tunnel_name.clone(),
                                TunnelStatus::Connected {
                                    connected_at: std::time::SystemTime::now(),
                                }
                            );
                            self.send_status_update(StatusUpdate::Connected(tunnel_name.clone()));
                            // logger.log_print(&format!("Tunnel '{}' started successfully (PID: {})", tunnel_name, process_id));
                        }
                        Err(_e) => {
                            // logger.log_print(&format!("Error checking tunnel status: {}", _e));
                        }
                    }
                }
                Ok(())
            }
            Err(e) => {
                // Set status to error
                let error = DrillError::SshProcess(e.to_string());
                self.tunnel_status.insert(
                    tunnel.name.clone(),
                    TunnelStatus::Error {
                        error: error.to_string(),
                        occurred_at: std::time::SystemTime::now(),
                    }
                );
                self.send_status_update(StatusUpdate::Error(tunnel.name.clone(), error.to_string()));
                // logger.log_print(&format!("Error starting tunnel '{}': {}", tunnel.name, e));
                Err(error)
            }
        }
    }
    

    /// Stop a tunnel
    pub fn stop_tunnel(&mut self, tunnel_name: &str) -> DrillResult<()> {
        if let Some(mut active) = self.active_processes.remove(tunnel_name) {
            // Kill the process
            let _ = active.process.kill();
            // Set status to disconnected
            self.tunnel_status.insert(tunnel_name.to_string(), TunnelStatus::Disconnected);
            // Send status update
            self.send_status_update(StatusUpdate::Disconnected(tunnel_name.to_string()));
            // logger.log_print(&format!("Tunnel '{}' disconnected", tunnel_name));
        } else {
            // logger.log_print(&format!("Tunnel '{}' is not active", tunnel_name));
        }
        Ok(())
    }

    /// Remove a tunnel by name
    pub fn remove_tunnel(&mut self, tunnel_name: &str) -> DrillResult<()> {
        // First, stop the tunnel if it's active
        if self.is_tunnel_active(tunnel_name) {
            self.stop_tunnel(tunnel_name)?;
        }

        // Remove from tunnels list
        if let Some(index) = self.tunnels.iter().position(|t| t.name == tunnel_name) {
            self.tunnels.remove(index);
            // logger.log_print(&format!("Tunnel '{}' removed", tunnel_name));
            Ok(())
        } else {
            Err(DrillError::Tunnel(format!("Tunnel '{}' not found", tunnel_name)))
        }
    }

    /// Test SSH connection without creating a tunnel
    pub fn test_tunnel(tunnel: &Tunnel) -> DrillResult<String> {
        let remote = format!("{}@{}", tunnel.ssh_user, tunnel.ssh_host);
        
        // log_print(&format!("Testing SSH connection to {} on port {}", remote, tunnel.ssh_port));

        // Use ssh with -o BatchMode=yes to avoid interactive prompts
        // and -o ConnectTimeout=5 to timeout quickly
        let mut command = Command::new("ssh");
        
        // Add private key if provided
        if !tunnel.private_key.trim().is_empty() {
            command.arg("-i").arg(&tunnel.private_key);
        }
        
        command
            .arg("-o")
            .arg("BatchMode=yes")
            .arg("-o")
            .arg("ConnectTimeout=5")
            .arg("-p")
            .arg(&tunnel.ssh_port)
            .arg(&remote)
            .arg("echo")
            .arg("'SSH connection test successful'");

        // On Windows, suppress terminal window
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        match command.output() {
            Ok(output) => {
                if output.status.success() {
                    // logger.log_print(&format!("SSH connection test to {} succeeded", remote));
                    Ok("\u{2713} SSH connection successful! You can now create the tunnel.".to_string())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    // logger.log_print(&format!("SSH connection test to {} failed: {}", remote, stderr));
                    Err(DrillError::SshProcess(format!("SSH connection failed: {}", stderr.trim())))
                }
            }
            Err(e) => {
                // logger.log_print(&format!("Error testing SSH connection to {}: {}", remote, e));
                Err(DrillError::SshProcess(format!("Error testing SSH connection: {}", e)))
            }
        }
    }

    /// Clean up all active tunnels
    pub fn cleanup(&mut self) {
        for (_name, mut active) in self.active_processes.drain() {
            let _ = active.process.kill();
            // logger.log_print(&format!("Stopped tunnel '{}' during cleanup", name));
        }
    }
}

impl Drop for TunnelManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}
