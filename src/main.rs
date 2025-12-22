mod config;
mod logs;
mod systemtray;
mod about;

use clap::Parser;
use tray_icon::menu::MenuEvent;
use tao::event_loop::{EventLoop, ControlFlow};
use logs::log_print;

#[derive(Parser, Debug)]
#[command(name = "drill")]
#[command(about = "A multi-platform tunnel drilling application", long_about = None)]

struct Args {}
fn main() {
    
    log_print("Drill - Multi-Platform tunnel drilling Application");
    log_print(&format!("Platform: {}", get_platform()));
    log_print("");

    // Initialize configuration
    match config::init_config() {
        Ok(config_path) => {
            log_print(&format!("Configuration loaded from: {}", config_path.display()));
        }
        Err(e) => {
            log_print(&format!("Error initializing configuration: {}", e));
            std::process::exit(1);
        }
    }

    // Create event loop for tray icon
    let event_loop = EventLoop::new();

    // Initialize system tray
    let (_tray_icon, menu_ids) = match systemtray::init_tray() {
        Ok((icon, ids)) => (icon, ids),
        Err(e) => {
            log_print(&format!("Error initializing system tray: {}", e));
            std::process::exit(1);
        }
    };

    log_print("Drill initialized. Application running...");

    // Get menu event receiver
    let menu_channel = MenuEvent::receiver();

    // Run the event loop
    event_loop.run(move |_event, _event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Check for menu events
        if let Ok(event) = menu_channel.try_recv() {
            if event.id == menu_ids.about {
                log_print("About menu item clicked");
                about::show_about_window();
            } else if event.id == menu_ids.quit {
                log_print("Quit selected from tray menu");
                *control_flow = ControlFlow::Exit;
            }
        }
    });
}

pub fn get_platform() -> &'static str {
    #[cfg(target_os = "macos")]
    return "macOS";
    
    #[cfg(target_os = "windows")]
    return "Windows";
    
    #[cfg(target_os = "linux")]
    return "Linux";
    
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    return "Unknown";
}


