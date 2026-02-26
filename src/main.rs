#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod logs;
mod notifications;
mod systemtray;
mod tunnels;
mod windows;
mod error;

use app::App;

#[cfg(target_os = "linux")]
fn init_gtk() {
    // Initialize GTK for tray icons on Linux (tray-icon crate requires GTK be initialized)
    if let Err(e) = gtk::init() {
        eprintln!("Failed to initialize GTK: {}", e);
    }
}

#[cfg(not(target_os = "linux"))]
fn init_gtk() {}

fn main() -> iced::Result {
    // Initialize the notification system
    notifications::init_notifications();
    // Ensure GTK is initialized on platforms that need it (Linux)
    init_gtk();
    
    iced::daemon(App::title_fn, App::update_fn, App::view_fn)
        .subscription(App::subscription_fn)
        .run_with(|| {
            let (app, task) = App::new();
            (app, task)
        })
}


