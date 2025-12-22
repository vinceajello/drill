use tray_icon::{TrayIconBuilder, menu::{Menu, MenuItem, MenuId, PredefinedMenuItem, Submenu}, TrayIcon};
use crate::logs::log_print;
use crate::tunnels::{Tunnel, TunnelStatus, TunnelManager};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct TrayMenuIds {
    pub create: MenuId,
    pub about: MenuId,
    pub quit: MenuId,
    pub tunnel_connect: HashMap<String, MenuId>,
    pub tunnel_disconnect: HashMap<String, MenuId>,
}

/// Initialize the system tray icon with menu
pub fn init_tray(tunnels: &Vec<Tunnel>, tunnel_manager: &Arc<Mutex<TunnelManager>>) -> Result<(TrayIcon, TrayMenuIds), Box<dyn std::error::Error>> {
    // Create a simple menu
    let menu = Menu::new();

    let create_tunnel = MenuItem::new("Drill New Tunnel", true, None);
    menu.append(&create_tunnel)?;

    menu.append(&PredefinedMenuItem::separator())?;
    
    // Add tunnels with submenu for each tunnel
    let mut tunnel_connect_ids = HashMap::new();
    let mut tunnel_disconnect_ids = HashMap::new();
    
    let manager = tunnel_manager.lock().unwrap();
    
    for tunnel in tunnels {
        // Get current status
        let status = manager.get_tunnel_status(&tunnel.name);
        let display_name = get_tunnel_display_name(&tunnel.name, status);
        
        // Create submenu for each tunnel with status indicator
        let tunnel_submenu = Submenu::new(&display_name, true);
        
        // Only show Connect if not connected, only show Disconnect if connected
        match status {
            TunnelStatus::Disconnected | TunnelStatus::Error => {
                let connect_item = MenuItem::new("Connect", true, None);
                let connect_id = connect_item.id().clone();
                tunnel_connect_ids.insert(tunnel.name.clone(), connect_id);
                tunnel_submenu.append(&connect_item)?;
            },
            TunnelStatus::Connecting | TunnelStatus::Connected => {
                let disconnect_item = MenuItem::new("Disconnect", true, None);
                let disconnect_id = disconnect_item.id().clone();
                tunnel_disconnect_ids.insert(tunnel.name.clone(), disconnect_id);
                tunnel_submenu.append(&disconnect_item)?;
            }
        }
        
        menu.append(&tunnel_submenu)?;
    }
    
    drop(manager);
    
    // Add separator if there are tunnels
    if !tunnels.is_empty() {
        menu.append(&PredefinedMenuItem::separator())?;
    }
    
    let about_item = MenuItem::new("About Drill", true, None);
    let quit_item = MenuItem::new("Quit", true, None);

    let create_id = create_tunnel.id().clone();
    let about_id = about_item.id().clone();
    let quit_id = quit_item.id().clone();
    
    menu.append(&about_item)?;
    menu.append(&quit_item)?;

    // Create the tray icon with a default icon
    let icon = create_tray_icon();
    
    #[cfg(target_os = "macos")]
    let tray_icon = {
        TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Drill Application - Click to see menu")
            .with_icon(icon)
            .with_icon_as_template(true)  // This makes it adapt to light/dark mode on macOS
            .build()?
    };

    #[cfg(not(target_os = "macos"))]
    let tray_icon = {
        TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Drill Application - Click to see menu")
            .with_icon(icon)
            .build()?
    };

    // Return the tray icon and menu IDs to keep them alive
    Ok((tray_icon, TrayMenuIds { 
        about: about_id, 
        quit: quit_id, 
        create: create_id,
        tunnel_connect: tunnel_connect_ids,
        tunnel_disconnect: tunnel_disconnect_ids,
    }))
}

/// Update the tray menu with current tunnel states
pub fn update_tray_menu(tray_icon: &mut TrayIcon, tunnels: &Vec<Tunnel>, tunnel_manager: &Arc<Mutex<TunnelManager>>) -> Result<TrayMenuIds, Box<dyn std::error::Error>> {
    // Create new menu
    let menu = Menu::new();

    let create_tunnel = MenuItem::new("Drill New Tunnel", true, None);
    menu.append(&create_tunnel)?;

    menu.append(&PredefinedMenuItem::separator())?;
    
    // Add tunnels with submenu for each tunnel
    let mut tunnel_connect_ids = HashMap::new();
    let mut tunnel_disconnect_ids = HashMap::new();
    
    let manager = tunnel_manager.lock().unwrap();
    
    for tunnel in tunnels {
        // Get current status
        let status = manager.get_tunnel_status(&tunnel.name);
        let display_name = get_tunnel_display_name(&tunnel.name, status);
        
        // Create submenu for each tunnel with status indicator
        let tunnel_submenu = Submenu::new(&display_name, true);
        
        // Only show Connect if not connected, only show Disconnect if connected
        match status {
            TunnelStatus::Disconnected | TunnelStatus::Error => {
                let connect_item = MenuItem::new("Connect", true, None);
                let connect_id = connect_item.id().clone();
                tunnel_connect_ids.insert(tunnel.name.clone(), connect_id);
                tunnel_submenu.append(&connect_item)?;
            },
            TunnelStatus::Connecting | TunnelStatus::Connected => {
                let disconnect_item = MenuItem::new("Disconnect", true, None);
                let disconnect_id = disconnect_item.id().clone();
                tunnel_disconnect_ids.insert(tunnel.name.clone(), disconnect_id);
                tunnel_submenu.append(&disconnect_item)?;
            }
        }
        
        menu.append(&tunnel_submenu)?;
    }
    
    drop(manager);
    
    // Add separator if there are tunnels
    if !tunnels.is_empty() {
        menu.append(&PredefinedMenuItem::separator())?;
    }
    
    let about_item = MenuItem::new("About Drill", true, None);
    let quit_item = MenuItem::new("Quit", true, None);

    let create_id = create_tunnel.id().clone();
    let about_id = about_item.id().clone();
    let quit_id = quit_item.id().clone();
    
    menu.append(&about_item)?;
    menu.append(&quit_item)?;

    // Update the tray icon menu
    tray_icon.set_menu(Some(Box::new(menu)));

    // Return the new menu IDs
    Ok(TrayMenuIds { 
        about: about_id, 
        quit: quit_id, 
        create: create_id,
        tunnel_connect: tunnel_connect_ids,
        tunnel_disconnect: tunnel_disconnect_ids,
    })
}

/// Get status indicator for tunnel name
pub fn get_tunnel_display_name(name: &str, status: TunnelStatus) -> String {
    let indicator = match status {
        TunnelStatus::Disconnected => "○ ",  // Empty circle (gray/disconnected)
        TunnelStatus::Connecting => "◐ ",   // Half-filled circle (connecting)
        TunnelStatus::Connected => "● ",    // Filled circle (connected/green)
        TunnelStatus::Error => "✗ ",        // X mark (error/red)
    };
    format!("{}{}", indicator, name)
}

/// Create a monochromatic icon suitable for system tray
fn create_tray_icon() -> tray_icon::Icon {
    // Create a monochromatic icon suitable for macOS menu bar (template mode)
    // Using black/white for best template rendering
    let width = 22;  // macOS menu bar standard height
    let height = 22;
    let mut rgba = Vec::with_capacity(width * height * 4);
    
    for y in 0..height {
        for x in 0..width {
            // Create a simple "D" shape for Drill
            let is_in_shape = {
                // Simple circle pattern
                let center_x = 11.0;
                let center_y = 11.0;
                let distance = ((x as f32 - center_x).powi(2) + (y as f32 - center_y).powi(2)).sqrt();
                
                // Circle with radius 8
                distance < 8.0 && distance > 5.0
            };
            
            if is_in_shape {
                // Black for the icon shape (will be inverted by macOS in template mode)
                rgba.push(0);    // R
                rgba.push(0);    // G
                rgba.push(0);    // B
                rgba.push(255);  // A (fully opaque)
            } else {
                // Transparent background
                rgba.push(0);    // R
                rgba.push(0);    // G
                rgba.push(0);    // B
                rgba.push(0);    // A (transparent)
            }
        }
    }
    
    match tray_icon::Icon::from_rgba(rgba, width as u32, height as u32) {
        Ok(icon) => {
            icon
        },
        Err(e) => {
            log_print(&format!("Error creating icon: {}", e));
            panic!("Failed to create tray icon");
        }
    }
}
