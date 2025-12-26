
impl std::fmt::Display for DrillError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DrillError::Io(e) => write!(f, "IO error: {}", e),
            DrillError::Yaml(e) => write!(f, "YAML error: {}", e),
            DrillError::SshProcess(s) => write!(f, "SSH process error: {}", s),
            DrillError::Tunnel(s) => write!(f, "Tunnel error: {}", s),
            DrillError::Config(s) => write!(f, "Config error: {}", s),
            DrillError::Notification(s) => write!(f, "Notification error: {}", s),
        }
    }
}

impl std::error::Error for DrillError {}

impl From<std::io::Error> for DrillError {
    fn from(e: std::io::Error) -> Self {
        DrillError::Io(e)
    }
}

impl From<serde_yaml::Error> for DrillError {
    fn from(e: serde_yaml::Error) -> Self {
        DrillError::Yaml(e)
    }
}

pub type DrillResult<T> = Result<T, DrillError>;

#[derive(Debug)]
pub enum DrillError {
    Io(std::io::Error),
    Yaml(serde_yaml::Error),
    SshProcess(String),
    Tunnel(String),
    Config(String),
    Notification(String),
    // Unknown(String),
}


// Manual Clone implementation for DrillError, only for variants with cloneable data
