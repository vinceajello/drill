use crate::error::{DrillResult, DrillError};

#[cfg(not(target_os = "macos"))]
use notify_rust::{Notification, Timeout};

#[cfg(target_os = "macos")]
use std::sync::{Once, atomic::{AtomicBool, Ordering}};

#[cfg(target_os = "macos")]
static INIT: Once = Once::new();

#[cfg(target_os = "macos")]
static INIT_SUCCESS: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "macos")]
pub fn init_notifications() {
    INIT.call_once(|| {
        use mac_notification_sys::{get_bundle_identifier_or_default, set_application};
        
        let bundle = get_bundle_identifier_or_default("com.drill.app");
        
        match set_application(&bundle) {
            Ok(_) => {
                INIT_SUCCESS.store(true, Ordering::Relaxed);
            }
            Err(_e) => {}
        }
    });
}

#[cfg(not(target_os = "macos"))]
pub fn init_notifications() {
    // No initialization needed on other platforms
}

#[cfg(target_os = "macos")]
fn show_macos_notification(title: &str, body: &str) -> DrillResult<()> {
    use mac_notification_sys::send_notification;
    
    if !INIT_SUCCESS.load(Ordering::Relaxed) {
        return Err(DrillError::Notification("Notification system not properly initialized".to_string()));
    }
    
    send_notification(
        title,
        None,
        body,
        None,
    ).map_err(|e| DrillError::Notification(format!("macOS notification error: {}", e)))?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn show_desktop_notification(summary: &str, body: &str, icon: &str, timeout_ms: u32) -> DrillResult<()> {
    let mut notif = Notification::new();
    notif
        .appname("Drill")
        .summary(summary)
        .body(body)
        .timeout(Timeout::Milliseconds(timeout_ms));

    if !icon.is_empty() {
        notif.icon(icon);
    }

    match notif.show() {
        Ok(_) => Ok(()),
        Err(e) => {
            #[cfg(target_os = "linux")]
            {
                let mut cmd = std::process::Command::new("notify-send");
                cmd.arg(summary).arg(body);
                if !icon.is_empty() {
                    cmd.arg("-i").arg(icon);
                }
                cmd.arg("-t").arg(timeout_ms.to_string());
                if let Ok(_) = cmd.spawn() {
                    return Ok(());
                }
            }
            Err(DrillError::Notification(format!("Notification error: {}", e)))
        }
    }
}

pub fn notify_tunnel_connected(tunnel_name: &str) -> DrillResult<()> {
    #[cfg(target_os = "macos")]
    {
        show_macos_notification(
            "Tunnel Connected",
            &format!("Tunnel '{}' is now connected", tunnel_name)
        )?;
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        show_desktop_notification(
            "Drill - Tunnel Connected",
            &format!("Tunnel '{}' is now connected", tunnel_name),
            "network-wired",
            5000,
        )?;
    }
    Ok(())
}

pub fn notify_tunnel_disconnected(tunnel_name: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = show_macos_notification(
            "Tunnel Disconnected",
            &format!("Tunnel '{}' has been disconnected", tunnel_name)
        );
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        let _ = show_desktop_notification(
            "Drill - Tunnel Disconnected",
            &format!("Tunnel '{}' has been disconnected", tunnel_name),
            "network-offline",
            5000,
        );
    }
}

pub fn notify_tunnel_error(tunnel_name: &str, error_message: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = show_macos_notification(
            "Tunnel Error",
            &format!("Failed to connect tunnel '{}':\n{}", tunnel_name, error_message)
        );
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        let _ = show_desktop_notification(
            "Drill - Tunnel Error",
            &format!("Failed to connect tunnel '{}':\n{}", tunnel_name, error_message),
            "dialog-error",
            10000,
        );
    }
}

pub fn notify_tunnel_removed(tunnel_name: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = show_macos_notification(
            "Tunnel Removed",
            &format!("Tunnel '{}' has been removed", tunnel_name)
        );
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        let _ = show_desktop_notification(
            "Drill - Tunnel Removed",
            &format!("Tunnel '{}' has been removed", tunnel_name),
            "user-trash",
            5000,
        );
    }
}

pub fn notify_tunnel_created(tunnel_name: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = show_macos_notification(
            "Tunnel Created",
            &format!("Tunnel '{}' has been created successfully", tunnel_name)
        );
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        let _ = show_desktop_notification(
            "Drill - Tunnel Created",
            &format!("Tunnel '{}' has been created successfully", tunnel_name),
            "emblem-default",
            5000,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_notification() {
        let res = notify_tunnel_connected("test_tunnel");
        assert!(res.is_ok());
    }
}
