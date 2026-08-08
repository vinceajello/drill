pub mod about;
pub mod create_tunnel;

use create_tunnel::TunnelFormState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormMode {
    Create,
    Edit { tunnel_id: String },
}

impl Default for FormMode {
    fn default() -> Self {
        FormMode::Create
    }
}

#[derive(Debug, Clone)]
pub enum WindowType {
    About,
    TunnelForm(TunnelFormState),
}

impl WindowType {
    pub fn new_tunnel_form_create() -> Self {
        WindowType::TunnelForm(TunnelFormState {
            mode: FormMode::Create,
            name: String::new(),
            local_host: "127.0.0.1".to_string(),
            local_port: String::new(),
            remote_host: "127.0.0.1".to_string(),
            remote_port: String::new(),
            ssh_user: String::new(),
            ssh_host: String::new(),
            ssh_port: "22".to_string(),
            private_key: String::new(),
            web_url: String::new(),
            error_message: None,
            test_message: None,
        })
    }

    pub fn new_tunnel_form_edit(tunnel: &crate::tunnels::Tunnel) -> Self {
        WindowType::TunnelForm(TunnelFormState {
            mode: FormMode::Edit { tunnel_id: tunnel.id.clone() },
            name: tunnel.name.clone(),
            local_host: tunnel.local_host.clone(),
            local_port: tunnel.local_port.clone(),
            remote_host: tunnel.remote_host.clone(),
            remote_port: tunnel.remote_port.clone(),
            ssh_user: tunnel.ssh_user.clone(),
            ssh_host: tunnel.ssh_host.clone(),
            ssh_port: tunnel.ssh_port.clone(),
            private_key: tunnel.private_key.clone(),
            web_url: tunnel.web_url.clone().unwrap_or_default(),
            error_message: None,
            test_message: None,
        })
    }
}
