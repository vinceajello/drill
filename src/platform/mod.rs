use directories::ProjectDirs;
use crate::error::{DrillError, DrillResult};
#[cfg(target_os = "linux")]
use crate::tunnels::{Tunnel, TunnelStatus};
#[cfg(target_os = "linux")]
use crate::systemtray::TrayMenuIds;

#[cfg(target_os = "linux")]
pub trait TrayAdapter: Send + Sync {
    fn update_menu(&mut self, tunnels: &[Tunnel], statuses: &[(String, TunnelStatus)]) -> DrillResult<TrayMenuIds>;
}

pub fn get_project_dirs() -> DrillResult<ProjectDirs> {
    ProjectDirs::from("com", "drill", "drill")
        .ok_or_else(|| DrillError::Config("Could not resolve system project directories".to_string()))
}

pub fn set_clipboard_text(text: &str) -> DrillResult<()> {
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|e| DrillError::Clipboard(format!("Failed to initialize clipboard: {}", e)))?;
    clipboard
        .set_text(text)
        .map_err(|e| DrillError::Clipboard(format!("Failed to copy to clipboard: {}", e)))?;
    Ok(())
}

pub fn get_platform_name() -> &'static str {
    #[cfg(target_os = "macos")]
    return "macOS";

    #[cfg(target_os = "windows")]
    return "Windows";

    #[cfg(target_os = "linux")]
    return "Linux";

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    return "Unknown";
}

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_copy() {
        let test_text = "ssh -L 8080:127.0.0.1:80 -N user@host";
        let res = set_clipboard_text(test_text);
        assert!(res.is_ok());

        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if let Ok(text) = clipboard.get_text() {
                assert_eq!(text, test_text);
            }
        }
    }
}
