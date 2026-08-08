use thiserror::Error;

pub type DrillResult<T> = Result<T, DrillError>;

#[derive(Debug, Error)]
pub enum DrillError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("SSH process error: {0}")]
    SshProcess(String),

    #[error("Tunnel error: {0}")]
    Tunnel(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Notification error: {0}")]
    Notification(String),
}
