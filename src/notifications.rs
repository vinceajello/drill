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
        use mac_notification_sys::set_application;
        
        let bundle = "com.drill.app";
        
        match set_application(bundle) {
            Ok(_) => {
                INIT_SUCCESS.store(true, Ordering::Relaxed);
            }
            Err(_e) => {

            }
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
    
    match send_notification(
        &format!("drill - {}", &title),
        None,
        body,
        None,
    ) {
        Ok(_) => Ok(()),
        Err(_e) => {
            Err(DrillError::Notification(String::from("fail to send notification")))
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
        Notification::new()
            .summary("Drill - Tunnel Connected")
            .body(&format!("Tunnel '{}' is now connected", tunnel_name))
            .icon("network-wired")
            .timeout(Timeout::Milliseconds(5000))
            .show()
            .map_err(|e| DrillError::Notification(format!("Notification error: {}", e)))?;
    }
    Ok(())
}

pub fn notify_tunnel_disconnected(tunnel_name: &str) {
    // logger.log_print(&format!("Showing notification: Tunnel '{}' disconnected", tunnel_name));
    
    #[cfg(target_os = "macos")]
    {
        match show_macos_notification(
            "Tunnel Disconnected",
            &format!("Tunnel '{}' has been disconnected", tunnel_name)
        ) {
            Ok(_) => {},
            Err(_) => {},
            // Err(e) => logger.log_print(&format!("Error showing notification: {}", e)),
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
            match Notification::new()
                .summary("Drill - Tunnel Disconnected")
                .body(&format!("Tunnel '{}' has been disconnected", tunnel_name))
                .icon("network-offline")
                .timeout(Timeout::Milliseconds(5000))
                .show()
            {
                Ok(_) => {},
                Err(_) => {},
            }
    }
}

pub fn notify_tunnel_error(tunnel_name: &str, error_message: &str) {
    // logger.log_print(&format!("Showing notification: Tunnel '{}' error - {}", tunnel_name, error_message));
    
    #[cfg(target_os = "macos")]
    {
        match show_macos_notification(
            "Tunnel Error",
            &format!("Failed to connect tunnel '{}':\n{}", tunnel_name, error_message)
        ) {
            Ok(_) => {},
            Err(_) => {},
            // Err(e) => logger.log_print(&format!("Error showing notification: {}", e)),
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
            match Notification::new()
                .summary("Drill - Tunnel Error")
                .body(&format!("Failed to connect tunnel '{}':\n{}", tunnel_name, error_message))
                .icon("dialog-error")
                .timeout(Timeout::Milliseconds(10000))
                .show()
            {
                Ok(_) => {},
                Err(_) => {},
            }
    }
}

pub fn notify_tunnel_removed(tunnel_name: &str) {
    // logger.log_print(&format!("Showing notification: Tunnel '{}' removed", tunnel_name));
    
    #[cfg(target_os = "macos")]
    {
        match show_macos_notification(
            "Tunnel Removed",
            &format!("Tunnel '{}' has been removed", tunnel_name)
        ) {
            Ok(_) => {},
            Err(_) => {},
            // Err(e) => logger.log_print(&format!("Error showing notification: {}", e)),
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
            match Notification::new()
                .summary("Drill - Tunnel Removed")
                .body(&format!("Tunnel '{}' has been removed", tunnel_name))
                .icon("user-trash")
                .timeout(Timeout::Milliseconds(5000))
                .show()
            {
                Ok(_) => {},
                Err(_) => {},
            }
    }
}

pub fn notify_tunnel_created(tunnel_name: &str) {
    // logger.log_print(&format!("Showing notification: Tunnel '{}' created", tunnel_name));
    
    #[cfg(target_os = "macos")]
    {
        match show_macos_notification(
            "Tunnel Created",
            &format!("Tunnel '{}' has been created successfully", tunnel_name)
        ) {
            Ok(_) => {},
            Err(_) => {},
            // Err(e) => logger.log_print(&format!("Error showing notification: {}", e)),
        }
    }
    
    #[cfg(not(target_os = "macos"))]
    {
        match Notification::new()
            .summary("Drill - Tunnel Created")
            .body(&format!("Tunnel '{}' has been created successfully", tunnel_name))
            .icon("emblem-default")
            .timeout(Timeout::Milliseconds(5000))
            .show()
        {
            Ok(_) => {}
                Err(_) => {},
        }
    }
}
