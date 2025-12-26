use std::fs;
use std::path::PathBuf;
use std::io::{Write, BufRead, BufReader};
use std::process::{Command, Child, Stdio};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use crate::logs::log_print;

/// Custom error types for tunnel operations
#[derive(thiserror::Error, Debug, Clone)]
pub enum TunnelError {
    #[error("Process spawn failed: {0}")]
    ProcessSpawnFailed(String),
    
    #[error("Tunnel unexpectedly terminated: {0}")]
    UnexpectedTermination(String),
}

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
    Error(String, TunnelError),
    Disconnected(String),
}

/// Information about an active tunnel process
struct ActiveTunnel {
    process: Child,
    #[allow(dead_code)]
    started_at: Instant,
    monitor_tx: Option<tokio::sync::oneshot::Sender<()>>, // Signal to stop monitoring
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
    active_processes: Arc<Mutex<HashMap<String, ActiveTunnel>>>,
    tunnel_status: Arc<Mutex<HashMap<String, TunnelStatus>>>,
    status_tx: Arc<Mutex<Option<mpsc::UnboundedSender<StatusUpdate>>>>,
}

impl TunnelManager {
    pub fn new() -> Self {
        TunnelManager {
            tunnels: Vec::new(),
            active_processes: Arc::new(Mutex::new(HashMap::new())),
            tunnel_status: Arc::new(Mutex::new(HashMap::new())),
            status_tx: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Set the status update channel
    pub fn set_status_channel(&self, tx: mpsc::UnboundedSender<StatusUpdate>) {
        let mut status_tx = self.status_tx.lock().unwrap();
        *status_tx = Some(tx);
    }
    
    /// Send a status update
    fn send_status_update(&self, update: StatusUpdate) {
        if let Some(tx) = self.status_tx.lock().unwrap().as_ref() {
            let _ = tx.send(update);
        }
    }

    /// Load tunnels from the tunnels file
    pub fn load_tunnels(tunnels_file: &PathBuf) -> Result<Vec<Tunnel>, Box<dyn std::error::Error>> {
        if !tunnels_file.exists() {
            log_print("Tunnels file not found, returning empty list");
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(tunnels_file)?;
        let tunnels: Vec<Tunnel> = serde_yaml::from_str(&content)?;
        
        log_print(&format!("Loaded {} tunnel(s)", tunnels.len()));
        Ok(tunnels)
    }

    /// Save tunnels to the tunnels file
    pub fn save_tunnels(tunnels_file: &PathBuf, tunnels: &Vec<Tunnel>) -> Result<(), Box<dyn std::error::Error>> {
        let yaml = serde_yaml::to_string(tunnels)?;
        let mut file = fs::File::create(tunnels_file)?;
        file.write_all(yaml.as_bytes())?;
        
        log_print(&format!("Saved {} tunnel(s)", tunnels.len()));
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
    pub fn update_tunnel(&mut self, tunnel_id: &str, updated_tunnel: Tunnel) -> Result<(), Box<dyn std::error::Error>> {
        // Find tunnel by ID
        if let Some(index) = self.tunnels.iter().position(|t| t.id == tunnel_id) {
            // If tunnel is active, we may need to restart it with new settings
            let _old_name = self.tunnels[index].name.clone();
            
            // Update the tunnel
            self.tunnels[index] = updated_tunnel;
            
            log_print(&format!("Tunnel with ID '{}' updated", tunnel_id));
            Ok(())
        } else {
            Err(format!("Tunnel with ID '{}' not found", tunnel_id).into())
        }
    }

    /// Check if a tunnel is active
    pub fn is_tunnel_active(&self, tunnel_name: &str) -> bool {
        let processes = self.active_processes.lock().unwrap();
        processes.contains_key(tunnel_name)
    }

    /// Get the status of a tunnel
    pub fn get_tunnel_status(&self, tunnel_name: &str) -> TunnelStatus {
        let status = self.tunnel_status.lock().unwrap();
        status.get(tunnel_name).cloned().unwrap_or(TunnelStatus::Disconnected)
    }

    /// Start a tunnel with comprehensive error monitoring
    pub fn start_tunnel(&self, tunnel: &Tunnel) -> Result<(), Box<dyn std::error::Error>> {
        let mut processes = self.active_processes.lock().unwrap();
        
        if processes.contains_key(&tunnel.name) {
            log_print(&format!("Tunnel '{}' is already active", tunnel.name));
            return Ok(());
        }

        // Set status to connecting
        {
            let mut status = self.tunnel_status.lock().unwrap();
            status.insert(tunnel.name.clone(), TunnelStatus::Connecting);
        }
        
        // Send status update
        self.send_status_update(StatusUpdate::Connecting(tunnel.name.clone()));

        // Build SSH command with enhanced error detection
        let local_forward = format!(
            "{}:{}:{}",
            tunnel.local_port, tunnel.remote_host, tunnel.remote_port
        );
        let remote = format!("{}@{}", tunnel.ssh_user, tunnel.ssh_host);

        log_print(&format!(
            "Starting tunnel '{}': ssh -L {} -N -p {} {}",
            tunnel.name, local_forward, tunnel.ssh_port, remote
        ));

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

        match command.spawn() {
            Ok(mut child) => {
                let tunnel_name = tunnel.name.clone();
                let process_id = child.id();
                
                // Extract stderr for monitoring
                let stderr = child.stderr.take();
                
                // Create a channel for stopping the monitor
                let (stop_tx, stop_rx) = tokio::sync::oneshot::channel();
                
                // Store the process
                let active_tunnel = ActiveTunnel {
                    process: child,
                    started_at: Instant::now(),
                    monitor_tx: Some(stop_tx),
                };
                processes.insert(tunnel_name.clone(), active_tunnel);
                drop(processes);
                
                // Spawn monitoring task
                let status_map = Arc::clone(&self.tunnel_status);
                let active_processes = Arc::clone(&self.active_processes);
                let status_tx = Arc::clone(&self.status_tx);
                let tunnel_name_clone = tunnel_name.clone();
                
                tokio::spawn(async move {
                    Self::monitor_tunnel(
                        tunnel_name_clone,
                        process_id,
                        stderr,
                        status_map,
                        active_processes,
                        status_tx,
                        stop_rx,
                    ).await;
                });
                
                // Initial connection verification (give it a moment to start)
                std::thread::sleep(Duration::from_millis(500));
                
                // Check if process is still running
                let mut processes = self.active_processes.lock().unwrap();
                if let Some(active) = processes.get_mut(&tunnel_name) {
                    match active.process.try_wait() {
                        Ok(Some(status)) => {
                            // Process already exited
                            processes.remove(&tunnel_name);
                            drop(processes);
                            
                            let error = TunnelError::UnexpectedTermination(
                                format!("Process exited immediately with status: {}", status)
                            );
                            
                            let mut status_map = self.tunnel_status.lock().unwrap();
                            status_map.insert(
                                tunnel_name.clone(),
                                TunnelStatus::Error {
                                    error: error.to_string(),
                                    occurred_at: std::time::SystemTime::now(),
                                }
                            );
                            
                            self.send_status_update(StatusUpdate::Error(tunnel_name.clone(), error.clone()));
                            
                            log_print(&format!("Error starting tunnel '{}': {}", tunnel_name, error));
                            return Err(error.into());
                        }
                        Ok(None) => {
                            // Process is running - mark as connected
                            let mut status_map = self.tunnel_status.lock().unwrap();
                            status_map.insert(
                                tunnel_name.clone(),
                                TunnelStatus::Connected {
                                    connected_at: std::time::SystemTime::now(),
                                }
                            );
                            drop(status_map);
                            
                            self.send_status_update(StatusUpdate::Connected(tunnel_name.clone()));
                            log_print(&format!("Tunnel '{}' started successfully (PID: {})", tunnel_name, process_id));
                        }
                        Err(e) => {
                            log_print(&format!("Error checking tunnel status: {}", e));
                        }
                    }
                }
                
                Ok(())
            }
            Err(e) => {
                // Set status to error
                let error = TunnelError::ProcessSpawnFailed(e.to_string());
                let mut status = self.tunnel_status.lock().unwrap();
                status.insert(
                    tunnel.name.clone(),
                    TunnelStatus::Error {
                        error: error.to_string(),
                        occurred_at: std::time::SystemTime::now(),
                    }
                );
                drop(status);
                
                self.send_status_update(StatusUpdate::Error(tunnel.name.clone(), error.clone()));
                log_print(&format!("Error starting tunnel '{}': {}", tunnel.name, e));
                Err(error.into())
            }
        }
    }
    
    /// Monitor a tunnel process for errors and unexpected termination
    async fn monitor_tunnel(
        tunnel_name: String,
        process_id: u32,
        stderr: Option<std::process::ChildStderr>,
        status_map: Arc<Mutex<HashMap<String, TunnelStatus>>>,
        active_processes: Arc<Mutex<HashMap<String, ActiveTunnel>>>,
        status_tx: Arc<Mutex<Option<mpsc::UnboundedSender<StatusUpdate>>>>,
        mut stop_rx: tokio::sync::oneshot::Receiver<()>,
    ) {
        log_print(&format!("Starting monitor for tunnel '{}' (PID: {})", tunnel_name, process_id));
        
        // Spawn stderr reader task if available
        let stderr_handle = if let Some(stderr) = stderr {
            let tunnel_name_clone = tunnel_name.clone();
            Some(tokio::spawn(async move {
                Self::read_stderr(tunnel_name_clone, stderr).await
            }))
        } else {
            None
        };
        
        // Monitor loop
        let mut check_interval = tokio::time::interval(Duration::from_secs(5));
        
        loop {
            tokio::select! {
                _ = check_interval.tick() => {
                    // Check if process is still alive
                    let mut processes = active_processes.lock().unwrap();
                    
                    if let Some(active) = processes.get_mut(&tunnel_name) {
                        match active.process.try_wait() {
                            Ok(Some(exit_status)) => {
                                // Process has exited
                                log_print(&format!(
                                    "Tunnel '{}' process exited with status: {}",
                                    tunnel_name, exit_status
                                ));
                                
                                processes.remove(&tunnel_name);
                                drop(processes);
                                
                                let error = TunnelError::UnexpectedTermination(
                                    format!("Exit status: {}", exit_status)
                                );
                                
                                let mut status = status_map.lock().unwrap();
                                status.insert(
                                    tunnel_name.clone(),
                                    TunnelStatus::Error {
                                        error: error.to_string(),
                                        occurred_at: std::time::SystemTime::now(),
                                    }
                                );
                                drop(status);
                                
                                // Send status update
                                if let Some(tx) = status_tx.lock().unwrap().as_ref() {
                                    let _ = tx.send(StatusUpdate::Error(tunnel_name.clone(), error));
                                }
                                
                                break;
                            }
                            Ok(None) => {
                                // Process is still running - all good
                            }
                            Err(e) => {
                                log_print(&format!(
                                    "Error checking tunnel '{}' status: {}",
                                    tunnel_name, e
                                ));
                            }
                        }
                    } else {
                        // Tunnel was removed
                        break;
                    }
                }
                _ = &mut stop_rx => {
                    // Stop signal received
                    log_print(&format!("Monitor for tunnel '{}' received stop signal", tunnel_name));
                    break;
                }
            }
        }
        
        // Clean up stderr reader
        if let Some(handle) = stderr_handle {
            handle.abort();
        }
        
        log_print(&format!("Monitor for tunnel '{}' stopped", tunnel_name));
    }
    
    /// Read and parse SSH stderr for error messages
    async fn read_stderr(tunnel_name: String, stderr: std::process::ChildStderr) {
        let reader = BufReader::new(stderr);
        
        for line in reader.lines() {
            match line {
                Ok(line) => {
                    // Log SSH verbose output
                    log_print(&format!("SSH [{}]: {}", tunnel_name, line));
                    
                    // Parse common SSH error patterns
                    let lower = line.to_lowercase();
                    if lower.contains("permission denied") || lower.contains("authentication failed") {
                        log_print(&format!("Authentication error detected for tunnel '{}'", tunnel_name));
                    } else if lower.contains("connection refused") || lower.contains("connection timed out") {
                        log_print(&format!("Connection error detected for tunnel '{}'", tunnel_name));
                    } else if lower.contains("bind") && lower.contains("address already in use") {
                        log_print(&format!("Port already in use for tunnel '{}'", tunnel_name));
                    } else if lower.contains("could not resolve hostname") {
                        log_print(&format!("DNS resolution error for tunnel '{}'", tunnel_name));
                    }
                }
                Err(e) => {
                    log_print(&format!("Error reading stderr for tunnel '{}': {}", tunnel_name, e));
                    break;
                }
            }
        }
    }

    /// Stop a tunnel
    pub fn stop_tunnel(&self, tunnel_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut processes = self.active_processes.lock().unwrap();
        
        if let Some(mut active) = processes.remove(tunnel_name) {
            // Signal monitor to stop
            if let Some(tx) = active.monitor_tx.take() {
                let _ = tx.send(());
            }
            
            // Kill the process
            active.process.kill()?;
            
            // Set status to disconnected
            let mut status = self.tunnel_status.lock().unwrap();
            status.insert(tunnel_name.to_string(), TunnelStatus::Disconnected);
            drop(status);
            
            // Send status update
            self.send_status_update(StatusUpdate::Disconnected(tunnel_name.to_string()));
            
            log_print(&format!("Tunnel '{}' disconnected", tunnel_name));
        } else {
            log_print(&format!("Tunnel '{}' is not active", tunnel_name));
        }

        Ok(())
    }

    /// Remove a tunnel by name
    pub fn remove_tunnel(&mut self, tunnel_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        // First, stop the tunnel if it's active
        if self.is_tunnel_active(tunnel_name) {
            self.stop_tunnel(tunnel_name)?;
        }

        // Remove from tunnels list
        if let Some(index) = self.tunnels.iter().position(|t| t.name == tunnel_name) {
            self.tunnels.remove(index);
            log_print(&format!("Tunnel '{}' removed", tunnel_name));
            Ok(())
        } else {
            Err(format!("Tunnel '{}' not found", tunnel_name).into())
        }
    }

    /// Test SSH connection without creating a tunnel
    pub fn test_tunnel(tunnel: &Tunnel) -> Result<String, String> {
        let remote = format!("{}@{}", tunnel.ssh_user, tunnel.ssh_host);
        
        log_print(&format!(
            "Testing SSH connection to {} on port {}",
            remote, tunnel.ssh_port
        ));

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

        match command.output()
        {
            Ok(output) => {
                if output.status.success() {
                    log_print(&format!("SSH connection test to {} succeeded", remote));
                    Ok("✓ SSH connection successful! You can now create the tunnel.".to_string())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    log_print(&format!("SSH connection test to {} failed: {}", remote, stderr));
                    Err(format!("SSH connection failed: {}", stderr.trim()))
                }
            }
            Err(e) => {
                log_print(&format!("Error testing SSH connection to {}: {}", remote, e));
                Err(format!("Error testing SSH connection: {}", e))
            }
        }
    }

    /// Clean up all active tunnels
    pub fn cleanup(&self) {
        let mut processes = self.active_processes.lock().unwrap();
        for (name, mut active) in processes.drain() {
            // Signal monitor to stop
            if let Some(tx) = active.monitor_tx {
                let _ = tx.send(());
            }
            
            if let Err(e) = active.process.kill() {
                log_print(&format!("Error stopping tunnel '{}': {}", name, e));
            } else {
                log_print(&format!("Stopped tunnel '{}' during cleanup", name));
            }
        }
    }
}

impl Drop for TunnelManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}
