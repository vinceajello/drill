use crate::error::{DrillError, DrillResult};
use notify_rust::{Notification, Timeout};

pub fn init_notifications() {
    // Explicitly set the bundle identifier at startup so mac-notification-sys does not
    // fall back to querying "use_default" via AppleScript, which triggers the macOS
    // "Choose Application / Open With" prompt on the first notification.
    let bundle = mac_notification_sys::get_bundle_identifier_or_default("Drill");
    let _ = mac_notification_sys::set_application(&bundle);
}

pub fn show_macos_notification(title: &str, body: &str) -> DrillResult<()> {
    let mut notif = Notification::new();
    notif
        .appname("Drill")
        .summary(title)
        .body(body)
        .timeout(Timeout::Milliseconds(5000));

    notif
        .show()
        .map_err(|e| DrillError::Notification(format!("macOS notification error: {}", e)))?;
    Ok(())
}
