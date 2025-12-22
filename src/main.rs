mod config;
mod logs;
mod systemtray;
mod about;
mod tunnels;

use clap::Parser;
use tray_icon::menu::MenuEvent;
use tao::event_loop::{EventLoop, ControlFlow};
use logs::log_print;
use std::sync::{Arc, Mutex};

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

    // Load tunnels from the tunnels file
    let tunnels_file = match config::get_tunnels_file_path() {
        Ok(path) => path,
        Err(e) => {
            log_print(&format!("Error getting tunnels file path: {}", e));
            std::process::exit(1);
        }
    };

    let tunnels = match tunnels::TunnelManager::load_tunnels(&tunnels_file) {
        Ok(t) => t,
        Err(e) => {
            log_print(&format!("Error loading tunnels: {}", e));
            Vec::new()
        }
    };

    // Create tunnel manager
    let mut tunnel_manager = tunnels::TunnelManager::new();
    tunnel_manager.set_tunnels(tunnels.clone());
    let tunnel_manager = Arc::new(Mutex::new(tunnel_manager));

    // Create event loop for tray icon
    let event_loop = EventLoop::new();

    // Initialize system tray
    let (mut tray_icon, mut menu_ids) = match systemtray::init_tray(&tunnels, &tunnel_manager) {
        Ok((icon, ids)) => (icon, ids),
        Err(e) => {
            log_print(&format!("Error initializing system tray: {}", e));
            std::process::exit(1);
        }
    };

    log_print("Drill initialized. Application running...");

    // Get menu event receiver
    let menu_channel = MenuEvent::receiver();
    
    // Clone tunnels for the event loop
    let tunnels_for_loop = tunnels.clone();

    // Run the event loop
    event_loop.run(move |_event, _event_loop, control_flow| {
        *control_flow = ControlFlow::Wait;

        // Check for menu events
        if let Ok(event) = menu_channel.try_recv() {
            let mut should_update_menu = false;
            
            if event.id == menu_ids.create {
                log_print("Create menu item clicked");
                about::show_about_window();
            }
            if event.id == menu_ids.about {
                log_print("About menu item clicked");
                about::show_about_window();
            }
            if event.id == menu_ids.quit {
                log_print("Quit selected from tray menu");
                
                // Cleanup all tunnels before quitting
                let manager = tunnel_manager.lock().unwrap();
                manager.cleanup();
                drop(manager);
                
                *control_flow = ControlFlow::Exit;
            }

            // Check for tunnel connect events
            for (tunnel_name, connect_id) in &menu_ids.tunnel_connect {
                if event.id == *connect_id {
                    log_print(&format!("Connect tunnel '{}'", tunnel_name));
                    let manager = tunnel_manager.lock().unwrap();
                    if let Some(tunnel) = manager.get_tunnels().iter().find(|t| &t.name == tunnel_name) {
                        if let Err(e) = manager.start_tunnel(tunnel) {
                            log_print(&format!("Error starting tunnel '{}': {}", tunnel_name, e));
                        }
                        should_update_menu = true;
                    }
                    drop(manager);
                }
            }

            // Check for tunnel disconnect events
            for (tunnel_name, disconnect_id) in &menu_ids.tunnel_disconnect {
                if event.id == *disconnect_id {
                    log_print(&format!("Disconnect tunnel '{}'", tunnel_name));
                    let manager = tunnel_manager.lock().unwrap();
                    if let Err(e) = manager.stop_tunnel(tunnel_name) {
                        log_print(&format!("Error stopping tunnel '{}': {}", tunnel_name, e));
                    }
                    should_update_menu = true;
                    drop(manager);
                }
            }
            
            // Update menu if needed
            if should_update_menu {
                match systemtray::update_tray_menu(&mut tray_icon, &tunnels_for_loop, &tunnel_manager) {
                    Ok(new_ids) => {
                        menu_ids = new_ids;
                        log_print("Menu updated with new tunnel states");
                    },
                    Err(e) => {
                        log_print(&format!("Error updating menu: {}", e));
                    }
                }
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


