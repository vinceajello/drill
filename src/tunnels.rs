use std::fs;
use std::path::PathBuf;
use std::io::Write;
use std::process::{Command, Child};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::logs::log_print;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TunnelStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Tunnel {
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
    active_processes: Arc<Mutex<HashMap<String, Child>>>,
    tunnel_status: Arc<Mutex<HashMap<String, TunnelStatus>>>,
}

impl TunnelManager {
    pub fn new() -> Self {
        TunnelManager {
            tunnels: Vec::new(),
            active_processes: Arc::new(Mutex::new(HashMap::new())),
            tunnel_status: Arc::new(Mutex::new(HashMap::new())),
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

    /// Check if a tunnel is active
    pub fn is_tunnel_active(&self, tunnel_name: &str) -> bool {
        let processes = self.active_processes.lock().unwrap();
        processes.contains_key(tunnel_name)
    }

    /// Get the status of a tunnel
    pub fn get_tunnel_status(&self, tunnel_name: &str) -> TunnelStatus {
        let status = self.tunnel_status.lock().unwrap();
        status.get(tunnel_name).copied().unwrap_or(TunnelStatus::Disconnected)
    }

    /// Start a tunnel
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

        // Build SSH command: ssh -L local_port:remote_host:remote_port ssh_user@ssh_host -p ssh_port
        let local_forward = format!(
            "{}:{}:{}",
            tunnel.local_port, tunnel.remote_host, tunnel.remote_port
        );
        let remote = format!("{}@{}", tunnel.ssh_user, tunnel.ssh_host);

        let full_command = if !tunnel.private_key.trim().is_empty() {
            format!(
                "ssh -i {} -L {} -N -o ServerAliveInterval=60 -o ServerAliveCountMax=3 -p {} {}",
                tunnel.private_key, local_forward, tunnel.ssh_port, remote
            )
        } else {
            format!(
                "ssh -L {} -N -o ServerAliveInterval=60 -o ServerAliveCountMax=3 -p {} {}",
                local_forward, tunnel.ssh_port, remote
            )
        };

        log_print(&format!(
            "Starting tunnel '{}': {}",
            tunnel.name, full_command
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
            .arg("-o")
            .arg("ServerAliveInterval=60")
            .arg("-o")
            .arg("ServerAliveCountMax=3")
            .arg("-p")
            .arg(&tunnel.ssh_port)
            .arg(&remote);

        match command.spawn()
        {
            Ok(child) => {
                processes.insert(tunnel.name.clone(), child);
                // Set status to connected
                let mut status = self.tunnel_status.lock().unwrap();
                status.insert(tunnel.name.clone(), TunnelStatus::Connected);
                log_print(&format!("Tunnel '{}' started successfully", tunnel.name));
                Ok(())
            }
            Err(e) => {
                // Set status to error
                let mut status = self.tunnel_status.lock().unwrap();
                status.insert(tunnel.name.clone(), TunnelStatus::Error);
                log_print(&format!("Error starting tunnel '{}': {}", tunnel.name, e));
                Err(e.into())
            }
        }
    }

    /// Stop a tunnel
    pub fn stop_tunnel(&self, tunnel_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut processes = self.active_processes.lock().unwrap();
        
        if let Some(mut child) = processes.remove(tunnel_name) {
            child.kill()?;
            // Set status to disconnected
            let mut status = self.tunnel_status.lock().unwrap();
            status.insert(tunnel_name.to_string(), TunnelStatus::Disconnected);
            log_print(&format!("Tunnel '{}' stopped", tunnel_name));
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
        for (name, mut child) in processes.drain() {
            if let Err(e) = child.kill() {
                log_print(&format!("Error stopping tunnel '{}': {}", name, e));
            } else {
                log_print(&format!("Stopped tunnel '{}'", name));
            }
        }
    }
}

impl Drop for TunnelManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}
