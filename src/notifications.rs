pub fn init_notifications() {
    #[cfg(target_os = "macos")]
    crate::platform::macos::init_notifications();

    #[cfg(target_os = "linux")]
    crate::platform::linux::init_notifications();

    #[cfg(target_os = "windows")]
    crate::platform::windows::init_notifications();
}

pub fn notify_tunnel_connected(tunnel_name: &str) {
    let summary = "Drill - Tunnel Connected";
    let body = format!("Tunnel '{}' is now connected", tunnel_name);

    #[cfg(target_os = "macos")]
    let _ = crate::platform::macos::show_macos_notification(summary, &body);

    #[cfg(target_os = "linux")]
    let _ = crate::platform::linux::show_desktop_notification(summary, &body, "network-wired", 5000);

    #[cfg(target_os = "windows")]
    let _ = crate::platform::windows::show_desktop_notification(summary, &body, "network-wired", 5000);
}

pub fn notify_tunnel_disconnected(tunnel_name: &str) {
    let summary = "Drill - Tunnel Disconnected";
    let body = format!("Tunnel '{}' has been disconnected", tunnel_name);

    #[cfg(target_os = "macos")]
    let _ = crate::platform::macos::show_macos_notification(summary, &body);

    #[cfg(target_os = "linux")]
    let _ = crate::platform::linux::show_desktop_notification(summary, &body, "network-offline", 5000);

    #[cfg(target_os = "windows")]
    let _ = crate::platform::windows::show_desktop_notification(summary, &body, "network-offline", 5000);
}

pub fn notify_tunnel_error(tunnel_name: &str, error_message: &str) {
    let summary = "Drill - Tunnel Error";
    let body = format!("Failed to connect tunnel '{}':\n{}", tunnel_name, error_message);

    #[cfg(target_os = "macos")]
    let _ = crate::platform::macos::show_macos_notification(summary, &body);

    #[cfg(target_os = "linux")]
    let _ = crate::platform::linux::show_desktop_notification(summary, &body, "dialog-error", 10000);

    #[cfg(target_os = "windows")]
    let _ = crate::platform::windows::show_desktop_notification(summary, &body, "dialog-error", 10000);
}

pub fn notify_tunnel_removed(tunnel_name: &str) {
    let summary = "Drill - Tunnel Removed";
    let body = format!("Tunnel '{}' has been removed", tunnel_name);

    #[cfg(target_os = "macos")]
    let _ = crate::platform::macos::show_macos_notification(summary, &body);

    #[cfg(target_os = "linux")]
    let _ = crate::platform::linux::show_desktop_notification(summary, &body, "user-trash", 5000);

    #[cfg(target_os = "windows")]
    let _ = crate::platform::windows::show_desktop_notification(summary, &body, "user-trash", 5000);
}

pub fn notify_tunnel_created(tunnel_name: &str) {
    let summary = "Drill - Tunnel Created";
    let body = format!("Tunnel '{}' has been created successfully", tunnel_name);

    #[cfg(target_os = "macos")]
    let _ = crate::platform::macos::show_macos_notification(summary, &body);

    #[cfg(target_os = "linux")]
    let _ = crate::platform::linux::show_desktop_notification(summary, &body, "emblem-default", 5000);

    #[cfg(target_os = "windows")]
    let _ = crate::platform::windows::show_desktop_notification(summary, &body, "emblem-default", 5000);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_notification() {
        notify_tunnel_connected("test_tunnel");
    }
}
