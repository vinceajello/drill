use crate::error::{DrillError, DrillResult};
use notify_rust::{Notification, Timeout};

pub fn init_notifications() {
    // No explicit initialization required on macOS
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
