use crate::error::{DrillError, DrillResult};
use notify_rust::{Notification, Timeout};

pub fn init_notifications() {
    // No explicit initialization required on Windows
}

pub fn show_desktop_notification(summary: &str, body: &str, icon: &str, timeout_ms: u32) -> DrillResult<()> {
    let mut notif = Notification::new();
    notif
        .appname("Drill")
        .summary(summary)
        .body(body)
        .timeout(Timeout::Milliseconds(timeout_ms));

    if !icon.is_empty() {
        notif.icon(icon);
    }

    notif.show().map_err(|e| DrillError::Notification(format!("Windows notification error: {}", e)))?;
    Ok(())
}
