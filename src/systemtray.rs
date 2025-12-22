use tray_icon::{TrayIconBuilder, menu::{Menu, MenuItem, MenuId}, TrayIcon};
use crate::logs::log_print;

pub struct TrayMenuIds {
    pub about: MenuId,
    pub quit: MenuId,
}

/// Initialize the system tray icon with menu
pub fn init_tray() -> Result<(TrayIcon, TrayMenuIds), Box<dyn std::error::Error>> {
    // Create a simple menu
    let menu = Menu::new();
    let about_item = MenuItem::new("About Drill", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    
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

    // Return the tray icon and menu IDs to keep them alive
    Ok((tray_icon, TrayMenuIds { about: about_id, quit: quit_id }))
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
