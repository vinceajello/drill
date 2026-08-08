use crate::error::{DrillError, DrillResult};
use std::sync::{Once, atomic::{AtomicBool, Ordering}};

static INIT: Once = Once::new();
static INIT_SUCCESS: AtomicBool = AtomicBool::new(false);

pub fn init_notifications() {
    INIT.call_once(|| {
        use mac_notification_sys::{get_bundle_identifier_or_default, set_application};
        let bundle = get_bundle_identifier_or_default("com.drill.app");
        if set_application(&bundle).is_ok() {
            INIT_SUCCESS.store(true, Ordering::Relaxed);
        }
    });
}

pub fn show_macos_notification(title: &str, body: &str) -> DrillResult<()> {
    use mac_notification_sys::send_notification;

    if !INIT_SUCCESS.load(Ordering::Relaxed) {
        return Err(DrillError::Notification("macOS Notification system not initialized".to_string()));
    }

    send_notification(title, None, body, None)
        .map_err(|e| DrillError::Notification(format!("macOS notification error: {}", e)))?;
    Ok(())
}
