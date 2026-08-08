use crate::error::{DrillError, DrillResult};
use crate::systemtray::{update_tray_menu, init_tray, TrayMenuIds};
use crate::tunnels::{Tunnel, TunnelStatus};
use notify_rust::{Notification, Timeout};
use std::sync::mpsc;
use super::TrayAdapter;

pub fn init_notifications() {
    // No explicit initialization required on Linux
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

    match notif.show() {
        Ok(_) => Ok(()),
        Err(e) => {
            let mut cmd = std::process::Command::new("notify-send");
            cmd.arg(summary).arg(body);
            if !icon.is_empty() {
                cmd.arg("-i").arg(icon);
            }
            cmd.arg("-t").arg(timeout_ms.to_string());
            if cmd.spawn().is_ok() {
                return Ok(());
            }
            Err(DrillError::Notification(format!("Linux notification error: {}", e)))
        }
    }
}

pub enum LinuxTrayMsg {
    UpdateMenu {
        tunnels: Vec<Tunnel>,
        tunnel_statuses: Vec<(String, TunnelStatus)>,
        resp: mpsc::Sender<Result<TrayMenuIds, String>>,
    },
}

pub struct LinuxTrayHandle {
    sender: gtk::glib::Sender<LinuxTrayMsg>,
}

impl TrayAdapter for LinuxTrayHandle {
    fn update_menu(&mut self, tunnels: &[Tunnel], statuses: &[(String, TunnelStatus)]) -> DrillResult<TrayMenuIds> {
        let (resp_tx, resp_rx) = mpsc::channel();
        self.sender
            .send(LinuxTrayMsg::UpdateMenu {
                tunnels: tunnels.to_vec(),
                tunnel_statuses: statuses.to_vec(),
                resp: resp_tx,
            })
            .map_err(|e| DrillError::Tunnel(e.to_string()))?;

        let ids = resp_rx
            .recv()
            .map_err(|e| DrillError::Tunnel(e.to_string()))?
            .map_err(DrillError::Tunnel)?;
        Ok(ids)
    }
}

pub fn spawn_linux_tray(
    tunnels: Vec<Tunnel>,
    tunnel_statuses: Vec<(String, TunnelStatus)>,
) -> Result<(LinuxTrayHandle, TrayMenuIds), Box<dyn std::error::Error>> {
    let (init_tx, init_rx) = mpsc::channel::<Result<TrayMenuIds, String>>();
    #[allow(deprecated)]
    let (sender, receiver) = gtk::glib::MainContext::channel::<LinuxTrayMsg>(gtk::glib::Priority::default());

    std::thread::spawn(move || {
        if let Err(e) = gtk::init() {
            let _ = init_tx.send(Err(format!("Failed to initialize GTK: {}", e)));
            return;
        }

        let (mut tray_icon, initial_menu_ids) = match init_tray(&tunnels, &tunnel_statuses) {
            Ok((icon, ids)) => (icon, ids),
            Err(e) => {
                let _ = init_tx.send(Err(e.to_string()));
                return;
            }
        };

        let _ = tray_icon.set_visible(true);

        // Register with GNOME StatusNotifierWatcher DBus service
        let _ = std::process::Command::new("dbus-send")
            .args([
                "--session",
                "--dest=org.kde.StatusNotifierWatcher",
                "--type=method_call",
                "/StatusNotifierWatcher",
                "org.kde.StatusNotifierWatcher.RegisterStatusNotifierItem",
                "string:/org/ayatana/NotificationItem/drill",
            ])
            .output();

        let _ = init_tx.send(Ok(initial_menu_ids));

        // Event-driven channel receiver callback (no polling loop!)
        receiver.attach(None, move |msg| {
            match msg {
                LinuxTrayMsg::UpdateMenu { tunnels, tunnel_statuses, resp } => {
                    let res = update_tray_menu(&mut tray_icon, &tunnels, &tunnel_statuses)
                        .map_err(|e| e.to_string());
                    let _ = resp.send(res);
                }
            }
            gtk::glib::ControlFlow::Continue
        });

        gtk::main();
    });

    let menu_ids = init_rx.recv().map_err(|e| e.to_string())??;
    Ok((LinuxTrayHandle { sender }, menu_ids))
}
