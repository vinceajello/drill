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
        error_message: Option<String>,
    },
}

impl WindowType {
    pub fn new_create_tunnel() -> Self {
        WindowType::CreateTunnel {
            name: String::new(),
            local_host: "localhost".to_string(),
            local_port: String::new(),
            remote_host: String::new(),
            remote_port: String::new(),
            ssh_user: String::new(),
            ssh_host: String::new(),
            ssh_port: "22".to_string(),
            error_message: None,
        }
    }
}
