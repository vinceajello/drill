pub mod about;
pub mod create_tunnel;

#[derive(Debug, Clone)]
pub enum WindowType {
    About,
    CreateTunnel {
        name: String,
        local_host: String,
        local_port: String,
        remote_host: String,
        remote_port: String,
        ssh_user: String,
        ssh_host: String,
        ssh_port: String,
        private_key: String,
        error_message: Option<String>,
        test_message: Option<String>,
    },
    EditTunnel {
        tunnel_id: String,
        name: String,
        local_host: String,
        local_port: String,
        remote_host: String,
        remote_port: String,
        ssh_user: String,
        ssh_host: String,
        ssh_port: String,
        private_key: String,
        error_message: Option<String>,
        test_message: Option<String>,
    },
}

impl WindowType {
    pub fn new_create_tunnel() -> Self {
        WindowType::CreateTunnel {
            name: String::new(),
            local_host: "127.0.0.1".to_string(),
            local_port: String::new(),
            remote_host: "127.0.0.1".to_string(),
            remote_port: String::new(),
            ssh_user: String::new(),
            ssh_host: String::new(),
            ssh_port: "22".to_string(),
            private_key: String::new(),
            error_message: None,
            test_message: None,
        }
    }

    pub fn new_edit_tunnel(tunnel: &crate::tunnels::Tunnel) -> Self {
        WindowType::EditTunnel {
            tunnel_id: tunnel.id.clone(),
            name: tunnel.name.clone(),
            local_host: tunnel.local_host.clone(),
            local_port: tunnel.local_port.clone(),
            remote_host: tunnel.remote_host.clone(),
            remote_port: tunnel.remote_port.clone(),
            ssh_user: tunnel.ssh_user.clone(),
            ssh_host: tunnel.ssh_host.clone(),
            ssh_port: tunnel.ssh_port.clone(),
            private_key: tunnel.private_key.clone(),
            error_message: None,
            test_message: None,
        }
    }
}
