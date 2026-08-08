use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::net::TcpListener;
use tokio::sync::{broadcast, oneshot};
use tracing::{info, error};
use crate::error::{DrillResult, DrillError};

/// Enhanced tunnel status with timestamp details
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub web_url: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TunnelFile {
    pub tunnels: Vec<Tunnel>,
}

pub struct TunnelManager {
    tunnels: Vec<Tunnel>,
    cancel_txs: HashMap<String, oneshot::Sender<()>>,
    tunnel_status: HashMap<String, TunnelStatus>,
    status_tx: Option<broadcast::Sender<StatusUpdate>>,
}

impl TunnelManager {
    pub fn new() -> Self {
        TunnelManager {
            tunnels: Vec::new(),
            cancel_txs: HashMap::new(),
            tunnel_status: HashMap::new(),
            status_tx: None,
        }
    }

    pub fn set_status_channel(&mut self, tx: broadcast::Sender<StatusUpdate>) {
        self.status_tx = Some(tx);
    }

    fn send_status_update(&self, update: StatusUpdate) {
        if let Some(tx) = &self.status_tx {
            let _ = tx.send(update);
        }
    }

    pub fn load_tunnels(tunnels_file: &PathBuf) -> DrillResult<Vec<Tunnel>> {
        if !tunnels_file.exists() {
            return Ok(Vec::new());
        }
        let content = fs::read_to_string(tunnels_file)?;
        let file_data: TunnelFile = toml::from_str(&content)?;
        info!("Loaded {} tunnel(s) from TOML", file_data.tunnels.len());
        Ok(file_data.tunnels)
    }

    pub fn save_tunnels(tunnels_file: &PathBuf, tunnels: &[Tunnel]) -> DrillResult<()> {
        let file_data = TunnelFile {
            tunnels: tunnels.to_vec(),
        };
        let toml_str = toml::to_string_pretty(&file_data)?;
        fs::write(tunnels_file, toml_str)?;
        info!("Saved {} tunnel(s) to TOML", tunnels.len());
        Ok(())
    }

    pub fn set_tunnels(&mut self, tunnels: Vec<Tunnel>) {
        self.tunnels = tunnels;
    }

    pub fn get_tunnels(&self) -> &Vec<Tunnel> {
        &self.tunnels
    }

    pub fn add_tunnel(&mut self, tunnel: Tunnel) {
        self.tunnels.push(tunnel);
    }

    pub fn update_tunnel(&mut self, tunnel_id: &str, updated_tunnel: Tunnel) -> DrillResult<()> {
        if let Some(index) = self.tunnels.iter().position(|t| t.id == tunnel_id) {
            self.tunnels[index] = updated_tunnel;
            Ok(())
        } else {
            Err(DrillError::Tunnel(format!("Tunnel with ID '{}' not found", tunnel_id)))
        }
    }

    pub fn is_tunnel_active(&self, tunnel_name: &str) -> bool {
        self.cancel_txs.contains_key(tunnel_name)
    }

    pub fn get_tunnel_status(&self, tunnel_name: &str) -> TunnelStatus {
        self.tunnel_status.get(tunnel_name).cloned().unwrap_or(TunnelStatus::Disconnected)
    }

    pub fn update_status_from_event(&mut self, update: &StatusUpdate) {
        match update {
            StatusUpdate::Connecting(name) => {
                self.tunnel_status.insert(name.clone(), TunnelStatus::Connecting);
            }
            StatusUpdate::Connected(name) => {
                self.tunnel_status.insert(name.clone(), TunnelStatus::Connected {
                    connected_at: std::time::SystemTime::now(),
                });
            }
            StatusUpdate::Error(name, err) => {
                self.cancel_txs.remove(name);
                self.tunnel_status.insert(name.clone(), TunnelStatus::Error {
                    error: err.clone(),
                    occurred_at: std::time::SystemTime::now(),
                });
            }
            StatusUpdate::Disconnected(name) => {
                self.cancel_txs.remove(name);
                self.tunnel_status.insert(name.clone(), TunnelStatus::Disconnected);
            }
        }
    }

    pub fn start_tunnel(&mut self, tunnel: &Tunnel) -> DrillResult<()> {
        if self.cancel_txs.contains_key(&tunnel.name) {
            info!("Tunnel '{}' is already running", tunnel.name);
            return Ok(());
        }

        let Some(status_tx) = self.status_tx.clone() else {
            return Err(DrillError::Tunnel("Status broadcast channel not configured".to_string()));
        };

        let (cancel_tx, cancel_rx) = oneshot::channel();
        self.cancel_txs.insert(tunnel.name.clone(), cancel_tx);
        self.tunnel_status.insert(tunnel.name.clone(), TunnelStatus::Connecting);
        self.send_status_update(StatusUpdate::Connecting(tunnel.name.clone()));

        let tunnel_clone = tunnel.clone();
        tokio::spawn(async move {
            run_tunnel_supervisor(tunnel_clone, status_tx, cancel_rx).await;
        });

        Ok(())
    }

    pub fn stop_tunnel(&mut self, tunnel_name: &str) -> DrillResult<()> {
        if let Some(cancel_tx) = self.cancel_txs.remove(tunnel_name) {
            let _ = cancel_tx.send(());
            self.tunnel_status.insert(tunnel_name.to_string(), TunnelStatus::Disconnected);
            self.send_status_update(StatusUpdate::Disconnected(tunnel_name.to_string()));
            info!("Tunnel '{}' cancel signal sent", tunnel_name);
        }
        Ok(())
    }

    pub fn remove_tunnel(&mut self, tunnel_name: &str) -> DrillResult<()> {
        if self.is_tunnel_active(tunnel_name) {
            self.stop_tunnel(tunnel_name)?;
        }

        if let Some(index) = self.tunnels.iter().position(|t| t.name == tunnel_name) {
            self.tunnels.remove(index);
            Ok(())
        } else {
            Err(DrillError::Tunnel(format!("Tunnel '{}' not found", tunnel_name)))
        }
    }

    pub fn test_tunnel(tunnel: &Tunnel) -> DrillResult<String> {
        let remote = format!("{}@{}", tunnel.ssh_user, tunnel.ssh_host);
        let mut command = std::process::Command::new("ssh");

        if !tunnel.private_key.trim().is_empty() {
            check_private_key_permissions(&tunnel.private_key)?;
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
            .arg("SSH connection test successful");

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        match command.output() {
            Ok(output) => {
                if output.status.success() {
                    Ok("\u{2713} SSH connection successful! You can now create the tunnel.".to_string())
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(DrillError::SshProcess(format!("SSH connection failed: {}", stderr.trim())))
                }
            }
            Err(e) => Err(DrillError::SshProcess(format!("Error testing SSH connection: {}", e))),
        }
    }

    pub fn cleanup(&mut self) {
        for (name, cancel_tx) in self.cancel_txs.drain() {
            let _ = cancel_tx.send(());
            info!("Stopped tunnel '{}' during cleanup", name);
        }
    }
}

impl Drop for TunnelManager {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// Check if a local TCP port is available for binding
pub fn is_port_available(host: &str, port: u16) -> bool {
    TcpListener::bind((host, port)).is_ok()
}

/// Verify private key permissions on Unix platforms (warn if overly permissive)
pub fn check_private_key_permissions(path_str: &str) -> DrillResult<()> {
    let path = std::path::Path::new(path_str);
    if !path.exists() {
        return Err(DrillError::Config(format!("Private key file does not exist: {}", path_str)));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(path) {
            let mode = metadata.permissions().mode();
            // Check if group or others have read/write/execute permissions (0o077)
            if mode & 0o077 != 0 {
                tracing::warn!("Private key '{}' permissions ({:#o}) are overly permissive. Recommended: 0600 or 0400", path_str, mode & 0o777);
            }
        }
    }

    Ok(())
}

/// Async SSH tunnel process supervisor task
pub async fn run_tunnel_supervisor(
    tunnel: Tunnel,
    status_tx: broadcast::Sender<StatusUpdate>,
    mut cancel_rx: oneshot::Receiver<()>,
) {
    let port_num: u16 = match tunnel.local_port.parse() {
        Ok(p) => p,
        Err(_) => {
            let _ = status_tx.send(StatusUpdate::Error(
                tunnel.name.clone(),
                format!("Invalid local port number: {}", tunnel.local_port),
            ));
            return;
        }
    };

    let bind_host = if tunnel.local_host.trim().is_empty() {
        "127.0.0.1"
    } else {
        tunnel.local_host.trim()
    };

    if !is_port_available(bind_host, port_num) {
        let _ = status_tx.send(StatusUpdate::Error(
            tunnel.name.clone(),
            format!("Local port {}:{} is already in use", bind_host, port_num),
        ));
        return;
    }

    if !tunnel.private_key.trim().is_empty() {
        if let Err(e) = check_private_key_permissions(&tunnel.private_key) {
            let _ = status_tx.send(StatusUpdate::Error(tunnel.name.clone(), e.to_string()));
            return;
        }
    }

    let local_forward = format!(
        "{}:{}:{}:{}",
        bind_host, tunnel.local_port, tunnel.remote_host, tunnel.remote_port
    );
    let remote = format!("{}@{}", tunnel.ssh_user, tunnel.ssh_host);

    let mut cmd = tokio::process::Command::new("ssh");

    if !tunnel.private_key.trim().is_empty() {
        cmd.arg("-i").arg(&tunnel.private_key);
    }

    cmd.args([
        "-L",
        &local_forward,
        "-N",
        "-o",
        "ExitOnForwardFailure=yes",
        "-o",
        "ServerAliveInterval=30",
        "-o",
        "ServerAliveCountMax=3",
        "-o",
        "ConnectTimeout=10",
        "-p",
        &tunnel.ssh_port,
        &remote,
    ]);

    cmd.stderr(std::process::Stdio::piped());

    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let _ = status_tx.send(StatusUpdate::Error(
                tunnel.name.clone(),
                format!("Failed to spawn SSH process: {}", e),
            ));
            return;
        }
    };

    let stderr_pipe = child.stderr.take();

    info!("SSH tunnel supervisor spawned for '{}': ssh -L {} -N -p {} {}", tunnel.name, local_forward, tunnel.ssh_port, remote);
    let _ = status_tx.send(StatusUpdate::Connected(tunnel.name.clone()));

    tokio::select! {
        _ = &mut cancel_rx => {
            info!("Received cancellation for tunnel '{}'", tunnel.name);
            let _ = child.kill().await;
            let _ = status_tx.send(StatusUpdate::Disconnected(tunnel.name));
        }
        status = child.wait() => {
            let mut stderr_buf = String::new();
            if let Some(mut stderr) = stderr_pipe {
                let _ = tokio::io::AsyncReadExt::read_to_string(&mut stderr, &mut stderr_buf).await;
            }
            let stderr_clean = stderr_buf.trim();

            let err_msg = match status {
                Ok(exit_code) => {
                    if stderr_clean.is_empty() {
                        format!("SSH process exited with code {}", exit_code)
                    } else {
                        format!("SSH error (code {}): {}", exit_code, stderr_clean)
                    }
                }
                Err(e) => format!("SSH process wait error: {}", e),
            };
            error!("Tunnel '{}' ended unexpectedly: {}", tunnel.name, err_msg);
            let _ = status_tx.send(StatusUpdate::Error(tunnel.name, err_msg));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_port_available() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        assert!(!is_port_available("127.0.0.1", port));
        drop(listener);
        assert!(is_port_available("127.0.0.1", port));
    }

    #[test]
    fn test_tunnel_toml_serialization() {
        let tunnel = Tunnel {
            id: "123".to_string(),
            name: "Test Tunnel".to_string(),
            local_host: "127.0.0.1".to_string(),
            local_port: "8080".to_string(),
            remote_host: "10.0.0.1".to_string(),
            remote_port: "80".to_string(),
            ssh_user: "user".to_string(),
            ssh_host: "example.com".to_string(),
            ssh_port: "22".to_string(),
            private_key: "".to_string(),
            web_url: Some("http://localhost:8080".to_string()),
        };
        let file_data = TunnelFile {
            tunnels: vec![tunnel],
        };
        let toml_str = toml::to_string_pretty(&file_data).unwrap();
        assert!(toml_str.contains("Test Tunnel"));

        let deserialized: TunnelFile = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.tunnels.len(), 1);
        assert_eq!(deserialized.tunnels[0].name, "Test Tunnel");
    }
}
