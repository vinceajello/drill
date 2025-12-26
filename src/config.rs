use std::fs;
use std::path::PathBuf;
use std::io::Write;
use crate::logs::Logger;
use crate::error::{DrillResult, DrillError};

/// Initialize the application configuration directory and files
/// Returns the path to the config file and a Logger
pub fn init_config() -> DrillResult<(PathBuf, Logger)> {
    // Get home directory
    let home_dir = dirs::home_dir()
        .ok_or_else(|| DrillError::Config("Could not determine home directory".to_string()))?;
    
    // Create .drill directory path
    let drill_dir = home_dir.join(".drill");
    
    // Check if .drill directory exists, create if not
    if !drill_dir.exists() {
        println!("Creating .drill directory at: {}", drill_dir.display());
        fs::create_dir_all(&drill_dir)?;
        
        // Create logs directory
        let logs_dir = drill_dir.join("logs");
        println!("Creating logs directory at: {}", logs_dir.display());
        fs::create_dir_all(&logs_dir)?;
    }
    
    // Initialize log file
    let logs_dir = drill_dir.join("logs");
    if !logs_dir.exists() {
        fs::create_dir_all(&logs_dir)?;
    }
    
    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S");
    let log_file_path = logs_dir.join(format!("drill_{}.log", timestamp));
    let log_file = fs::File::create(&log_file_path)?;
    
    // Initialize the logger
    let mut logger = Logger::new(log_file);
    
    // Create config file path
    let config_file = drill_dir.join("config");
    
    // Check if config file exists, create if not
    if !config_file.exists() {
        logger.log_print(&format!("Creating default config file at: {}", config_file.display()));
        let mut file = fs::File::create(&config_file)?;
        // Write default configuration
        let default_config = r#"# Drill Configuration File
# Add your configuration settings here

[settings]
# Example setting
# key=value
"#;
        file.write_all(default_config.as_bytes())?;
    } else {
        logger.log_print(&format!("Config file found at: {}", config_file.display()));
        // Load existing config (for now just read it)
        let _config_content = fs::read_to_string(&config_file)?;
    }

    // Create tunnels file path
    let tunnels_file = drill_dir.join("tunnels");
    
    // Check if tunnels file exists, create if not
    if !tunnels_file.exists() {
        logger.log_print(&format!("Creating default tunnels file at: {}", tunnels_file.display()));
        let mut file = fs::File::create(&tunnels_file)?;
        // Write default empty tunnels array in YAML format
        let default_tunnels = "[]\n";
        file.write_all(default_tunnels.as_bytes())?;
    } else {
        logger.log_print(&format!("Tunnels file found at: {}", tunnels_file.display()));
    }
    Ok((config_file, logger))
}

/// Get the path to the tunnels file
pub fn get_tunnels_file_path() -> DrillResult<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| DrillError::Config("Could not determine home directory".to_string()))?;
    let drill_dir = home_dir.join(".drill");
    Ok(drill_dir.join("tunnels"))
}
