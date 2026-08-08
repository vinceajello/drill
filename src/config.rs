use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tracing::info;
use crate::error::DrillResult;
use crate::platform::get_project_dirs;

/// Initialize the application configuration directory and log files.
/// Returns (config_file_path, logs_dir_path).
pub fn init_config() -> DrillResult<(PathBuf, PathBuf)> {
    let proj_dirs = get_project_dirs()?;
    let config_dir = proj_dirs.config_dir();
    let data_dir = proj_dirs.data_local_dir();
    let logs_dir = data_dir.join("logs");

    // Ensure directories exist
    fs::create_dir_all(config_dir)?;
    fs::create_dir_all(&logs_dir)?;

    // Handle legacy ~/.drill migration if present BEFORE creating default empty tunnels.toml
    migrate_legacy_config(config_dir)?;

    let config_file = config_dir.join("config.toml");
    if !config_file.exists() {
        info!("Creating default configuration file at {}", config_file.display());
        let mut file = fs::File::create(&config_file)?;
        let default_config = r#"# Drill Configuration File

[settings]
server_alive_interval = 30
server_alive_count_max = 3
connect_timeout = 10
"#;
        file.write_all(default_config.as_bytes())?;
    } else {
        info!("Config file found at {}", config_file.display());
    }

    let tunnels_file = config_dir.join("tunnels.toml");
    if !tunnels_file.exists() {
        info!("Creating default tunnels file at {}", tunnels_file.display());
        let mut file = fs::File::create(&tunnels_file)?;
        file.write_all(b"tunnels = []\n")?;
    } else {
        info!("Tunnels file found at {}", tunnels_file.display());
    }

    Ok((config_file, logs_dir))
}

/// Get path to tunnels.toml file
pub fn get_tunnels_file_path() -> DrillResult<PathBuf> {
    let proj_dirs = get_project_dirs()?;
    Ok(proj_dirs.config_dir().join("tunnels.toml"))
}

/// Migrate legacy `~/.drill` configuration and tunnels if present
fn migrate_legacy_config(target_config_dir: &std::path::Path) -> DrillResult<()> {
    let home_dir = match dirs::home_dir() {
        Some(h) => h,
        None => return Ok(()),
    };

    let legacy_dir = home_dir.join(".drill");
    if !legacy_dir.exists() {
        return Ok(());
    }

    let legacy_tunnels = legacy_dir.join("tunnels");
    let target_tunnels = target_config_dir.join("tunnels.toml");

    // Check if target tunnels doesn't exist or is currently empty
    let should_migrate = if !target_tunnels.exists() {
        true
    } else if let Ok(existing_content) = fs::read_to_string(&target_tunnels) {
        if let Ok(file_data) = toml::from_str::<crate::tunnels::TunnelFile>(&existing_content) {
            file_data.tunnels.is_empty()
        } else {
            true
        }
    } else {
        true
    };

    if legacy_tunnels.exists() && should_migrate {
        info!("Migrating legacy configuration from {}", legacy_dir.display());
        if let Ok(content) = fs::read_to_string(&legacy_tunnels) {
            // Attempt to parse legacy YAML or JSON tunnels
            let tunnels_res = serde_yaml::from_str::<Vec<crate::tunnels::Tunnel>>(&content)
                .or_else(|_| serde_json::from_str::<Vec<crate::tunnels::Tunnel>>(&content));

            if let Ok(tunnels) = tunnels_res {
                if !tunnels.is_empty() {
                    let toml_struct = crate::tunnels::TunnelFile { tunnels };
                    if let Ok(toml_str) = toml::to_string_pretty(&toml_struct) {
                        let _ = fs::write(&target_tunnels, toml_str);
                        info!("Successfully migrated {} legacy tunnels to TOML at {}", toml_struct.tunnels.len(), target_tunnels.display());
                    }
                }
            }
        }
    }

    Ok(())
}
