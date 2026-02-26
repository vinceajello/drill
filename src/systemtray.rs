use tray_icon::{TrayIconBuilder, menu::{Menu, MenuItem, MenuId, PredefinedMenuItem, Submenu}, TrayIcon};
use crate::tunnels::{Tunnel, TunnelStatus};
use std::collections::HashMap;

pub struct TrayMenuIds {
    pub create: MenuId,
    pub about: MenuId,
    pub quit: MenuId,
    pub tunnel_connect: HashMap<String, MenuId>,
    pub tunnel_disconnect: HashMap<String, MenuId>,
    pub tunnel_open_web: HashMap<String, MenuId>,
    pub tunnel_edit: HashMap<String, MenuId>,
    pub tunnel_remove: HashMap<String, MenuId>,
}

/// Initialize the system tray icon with menu
pub fn init_tray(tunnels: &Vec<Tunnel>, tunnel_statuses: &[(String, TunnelStatus)]) -> Result<(TrayIcon, TrayMenuIds), Box<dyn std::error::Error>> {
    // Create a simple menu
    let menu = Menu::new();

    let create_tunnel = MenuItem::new("Drill New Tunnel", true, None);
    menu.append(&create_tunnel)?;

    menu.append(&PredefinedMenuItem::separator())?;
    
    // Add tunnels with submenu for each tunnel
    let mut tunnel_connect_ids = HashMap::new();
    let mut tunnel_disconnect_ids = HashMap::new();
    let mut tunnel_open_web_ids: HashMap<String, MenuId> = HashMap::new();
    let mut tunnel_edit_ids = HashMap::new();
    let mut tunnel_remove_ids = HashMap::new();
    
    let status_map: std::collections::HashMap<_, _> = tunnel_statuses.iter().cloned().collect();
    for tunnel in tunnels {
        // Get current status
        let status = status_map.get(&tunnel.name).cloned().unwrap_or(TunnelStatus::Disconnected);
        let display_name = get_tunnel_display_name(&tunnel.name, status.clone());
        
        // Create submenu for each tunnel with status indicator
        let tunnel_submenu = Submenu::new(&display_name, true);
        
        // Only show Connect if not connected, only show Disconnect if connected
        match &status {
            TunnelStatus::Disconnected | TunnelStatus::Error { .. } => {
                let connect_item = MenuItem::new("Connect", true, None);
                let connect_id = connect_item.id().clone();
                tunnel_connect_ids.insert(tunnel.name.clone(), connect_id);
                tunnel_submenu.append(&connect_item)?;
            },
            TunnelStatus::Connecting | TunnelStatus::Connected { .. } | TunnelStatus::Reconnecting { .. } => {
                let disconnect_item = MenuItem::new("Disconnect", true, None);
                let disconnect_id = disconnect_item.id().clone();
                tunnel_disconnect_ids.insert(tunnel.name.clone(), disconnect_id);
                tunnel_submenu.append(&disconnect_item)?;
                
                let status = tunnel_statuses.iter().find(|(name, _)| name == &tunnel.name).map(|(_, status)| status.clone()).unwrap_or(TunnelStatus::Disconnected);
                if matches!(status, TunnelStatus::Connected { .. }) {
                    let open_web_item = MenuItem::new("Open Web", true, None);
                    let open_web_id = open_web_item.id().clone();
                    tunnel_open_web_ids.insert(tunnel.name.clone(), open_web_id);
                    tunnel_submenu.append(&open_web_item)?;
                }
            }
        }
        
        // Add Edit option (disabled when connected)
        let is_connected = matches!(status, TunnelStatus::Connecting | TunnelStatus::Connected { .. } | TunnelStatus::Reconnecting { .. });
        let edit_item = MenuItem::new("Edit", !is_connected, None);
        let edit_id = edit_item.id().clone();
        tunnel_edit_ids.insert(tunnel.name.clone(), edit_id);
        tunnel_submenu.append(&edit_item)?;
        
        // Add Remove option (disabled when connected)
        let remove_item = MenuItem::new("Remove", !is_connected, None);
        let remove_id = remove_item.id().clone();
        tunnel_remove_ids.insert(tunnel.name.clone(), remove_id);
        tunnel_submenu.append(&remove_item)?;
        
        menu.append(&tunnel_submenu)?;
    }
    
    
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
        tunnel_open_web: tunnel_open_web_ids,
        tunnel_edit: tunnel_edit_ids,
        tunnel_remove: tunnel_remove_ids,
    }))
}

/// Update the tray menu with current tunnel states
pub fn update_tray_menu(tray_icon: &mut TrayIcon, tunnels: &Vec<Tunnel>, tunnel_statuses: &[(String, TunnelStatus)]) -> Result<TrayMenuIds, Box<dyn std::error::Error>> {
    // Create new menu
    let menu = Menu::new();

    let create_tunnel = MenuItem::new("Drill New Tunnel", true, None);
    menu.append(&create_tunnel)?;

    menu.append(&PredefinedMenuItem::separator())?;
    
    // Add tunnels with submenu for each tunnel
    let mut tunnel_connect_ids = HashMap::new();
    let mut tunnel_disconnect_ids = HashMap::new();
    let mut tunnel_open_web_ids: HashMap<String, MenuId> = HashMap::new();
    let mut tunnel_edit_ids = HashMap::new();
    let mut tunnel_remove_ids = HashMap::new();
    
    for tunnel in tunnels {
        // Get current status from tunnel_statuses
        let status = tunnel_statuses.iter().find(|(name, _)| name == &tunnel.name).map(|(_, status)| status.clone()).unwrap_or(TunnelStatus::Disconnected);
        let display_name = get_tunnel_display_name(&tunnel.name, status.clone());
        
        // Create submenu for each tunnel with status indicator
        let tunnel_submenu = Submenu::new(&display_name, true);
        
        // Only show Connect if not connected, only show Disconnect if connected
        match &status {
            TunnelStatus::Disconnected | TunnelStatus::Error { .. } => {
                let connect_item = MenuItem::new("Connect", true, None);
                let connect_id = connect_item.id().clone();
                tunnel_connect_ids.insert(tunnel.name.clone(), connect_id);
                tunnel_submenu.append(&connect_item)?;
            },
            TunnelStatus::Connecting | TunnelStatus::Connected { .. } | TunnelStatus::Reconnecting { .. } => {
                let disconnect_item = MenuItem::new("Disconnect", true, None);
                let disconnect_id = disconnect_item.id().clone();
                tunnel_disconnect_ids.insert(tunnel.name.clone(), disconnect_id);
                tunnel_submenu.append(&disconnect_item)?;
                
                // Add "Open Web" button when connected
                if matches!(status, TunnelStatus::Connected { .. }) {
                    let open_web_item = MenuItem::new("Open Web", true, None);
                    let open_web_id = open_web_item.id().clone();
                    tunnel_open_web_ids.insert(tunnel.name.clone(), open_web_id);
                    tunnel_submenu.append(&open_web_item)?;
                }
            }
        }
        
        // Add Edit option (disabled when connected)
        let is_connected = matches!(status, TunnelStatus::Connecting | TunnelStatus::Connected { .. } | TunnelStatus::Reconnecting { .. });
        let edit_item = MenuItem::new("Edit", !is_connected, None);
        let edit_id = edit_item.id().clone();
        tunnel_edit_ids.insert(tunnel.name.clone(), edit_id);
        tunnel_submenu.append(&edit_item)?;
        
        // Add Remove option (disabled when connected)
        let remove_item = MenuItem::new("Remove", !is_connected, None);
        let remove_id = remove_item.id().clone();
        tunnel_remove_ids.insert(tunnel.name.clone(), remove_id);
        tunnel_submenu.append(&remove_item)?;
        
        menu.append(&tunnel_submenu)?;
    }
    
    // No manager to drop
    
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
        tunnel_open_web: tunnel_open_web_ids,
        tunnel_edit: tunnel_edit_ids,
        tunnel_remove: tunnel_remove_ids,
    })
}

/// Get status indicator for tunnel name
pub fn get_tunnel_display_name(name: &str, status: TunnelStatus) -> String {
    let indicator = match status {
        TunnelStatus::Disconnected => "○ ",  // Empty circle (gray/disconnected)
        TunnelStatus::Connecting => "◐ ",   // Half-filled circle (connecting)
        TunnelStatus::Connected { .. } => "● ",    // Filled circle (connected/green)
        TunnelStatus::Error { .. } => "✗ ",        // X mark (error/red)
        TunnelStatus::Reconnecting { .. } => "↻ ",  // Refresh/reconnecting
    };
    format!("{}{}", indicator, name)
}

/// Create a monochromatic icon suitable for system tray
fn create_tray_icon() -> tray_icon::Icon {
    // Create a monochromatic icon suitable for macOS menu bar (template mode)
    // Using black/white for best template rendering
    let width = 32;
    let height = 32;
    let mut rgba = Vec::with_capacity(width * height * 4);

    // Produce a solid, colored circular icon (green fill with slight border)
    let cx = (width / 2) as f32;
    let cy = (height / 2) as f32;
    let radius = (width as f32) * 0.38; // circle radius
    let border = (width as f32) * 0.08; // border thickness

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist <= radius {
                // Inside circle: green fill
                rgba.push(0x1a); // R (26)
                rgba.push(0xa0); // G (160)
                rgba.push(0x2a); // B (42)
                rgba.push(0xff); // A
            } else if dist <= radius + border {
                // Slight outer border: darker green
                rgba.push(0x10);
                rgba.push(0x70);
                rgba.push(0x20);
                rgba.push(0xff);
            } else {
                // Transparent background
                rgba.push(0x00);
                rgba.push(0x00);
                rgba.push(0x00);
                rgba.push(0x00);
            }
        }
    }
    
    match tray_icon::Icon::from_rgba(rgba, width as u32, height as u32) {
        Ok(icon) => {
            icon
        },
        Err(_e) => {
            // log_print(&format!("Error creating icon: {}", _e));
            panic!("Failed to create tray icon");
        }
    }
}
