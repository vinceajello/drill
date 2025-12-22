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
    pub user: String,
    pub host: String,
    pub r_host: String,
    pub r_port: u16,
    pub l_host: String,
    pub l_port: u16,
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

        // Build SSH command: ssh -L l_port:r_host:r_port user@host
        let local_forward = format!("{}:{}:{}", tunnel.l_port, tunnel.r_host, tunnel.r_port);
        let remote = format!("{}@{}", tunnel.user, tunnel.host);

        log_print(&format!("Starting tunnel '{}': ssh -L {} -N {}", 
            tunnel.name, local_forward, remote));

        match Command::new("ssh")
            .arg("-L")
            .arg(&local_forward)
            .arg("-N")  // Don't execute remote command
            .arg("-o")
            .arg("ServerAliveInterval=60")
            .arg("-o")
            .arg("ServerAliveCountMax=3")
            .arg(&remote)
            .spawn() {
                Ok(child) => {
                    processes.insert(tunnel.name.clone(), child);
                    // Set status to connected
                    let mut status = self.tunnel_status.lock().unwrap();
                    status.insert(tunnel.name.clone(), TunnelStatus::Connected);
                    log_print(&format!("Tunnel '{}' started successfully", tunnel.name));
                    Ok(())
                },
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

    /// Toggle a tunnel (start if stopped, stop if started)
    pub fn toggle_tunnel(&self, tunnel_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_tunnel_active(tunnel_name) {
            self.stop_tunnel(tunnel_name)?;
        } else {
            if let Some(tunnel) = self.tunnels.iter().find(|t| t.name == tunnel_name) {
                self.start_tunnel(tunnel)?;
            } else {
                return Err(format!("Tunnel '{}' not found", tunnel_name).into());
            }
        }
        Ok(())
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
