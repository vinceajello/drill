use crate::config;
use crate::logs::log_print;
use crate::notifications;
use crate::systemtray::{self, TrayMenuIds};
use crate::tunnels::TunnelManager;
use crate::windows::{self, WindowType};
use iced::futures::SinkExt;
use iced::window;
use iced::{Element, Size, Subscription, Task};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tray_icon::menu::MenuEvent;
use tray_icon::TrayIcon;

pub struct App {
    windows: BTreeMap<window::Id, WindowType>,
    tunnel_manager: Arc<Mutex<TunnelManager>>,
    tunnels_file: PathBuf,
    tray_icon: Option<TrayIcon>,
    menu_ids: Option<TrayMenuIds>,
}

#[derive(Debug, Clone)]
pub enum Message {
    // Tray menu events
    TrayMenuEvent(MenuEvent),
    OpenAbout,
    OpenCreateTunnel,
    TunnelConnect(String),
    TunnelDisconnect(String),
    TunnelEdit(String),
    TunnelRemove(String),
    Quit,

    // Window events
    WindowOpened(window::Id, WindowType),
    WindowClosed(window::Id),

    // About window messages
    //AboutClose,

    // Create tunnel window messages
    CreateTunnelNameChanged(String),
    CreateTunnelLocalHostChanged(String),
    CreateTunnelLocalPortChanged(String),
    CreateTunnelRemoteHostChanged(String),
    CreateTunnelRemotePortChanged(String),
    CreateTunnelSshUserChanged(String),
    CreateTunnelSshHostChanged(String),
    CreateTunnelSshPortChanged(String),
    CreateTunnelPrivateKeyChanged(String),
    CreateTunnelBrowsePrivateKey,
    CreateTunnelTest(window::Id),
    CreateTunnelCreate(window::Id),
    CreateTunnelCancel(window::Id),

    // Edit tunnel window messages
    EditTunnelNameChanged(String),
    EditTunnelLocalHostChanged(String),
    EditTunnelLocalPortChanged(String),
    EditTunnelRemoteHostChanged(String),
    EditTunnelRemotePortChanged(String),
    EditTunnelSshUserChanged(String),
    EditTunnelSshHostChanged(String),
    EditTunnelSshPortChanged(String),
    EditTunnelPrivateKeyChanged(String),
    EditTunnelBrowsePrivateKey,
    EditTunnelTest(window::Id),
    EditTunnelSave(window::Id),
    EditTunnelCancel(window::Id),

    // Internal
    UpdateTrayMenu,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        log_print("Drill - Multi-Platform tunnel drilling Application");
        log_print(&format!("Platform: {}", get_platform()));
        log_print("");

        // Initialize configuration
        match config::init_config() {
            Ok(config_path) => {
                log_print(&format!(
                    "Configuration loaded from: {}",
                    config_path.display()
                ));
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

        let tunnels = match TunnelManager::load_tunnels(&tunnels_file) {
            Ok(t) => t,
            Err(e) => {
                log_print(&format!("Error loading tunnels: {}", e));
                Vec::new()
            }
        };

        // Create tunnel manager
        let mut tunnel_manager = TunnelManager::new();
        tunnel_manager.set_tunnels(tunnels.clone());
        let tunnel_manager = Arc::new(Mutex::new(tunnel_manager));

        // Initialize system tray
        let (tray_icon, menu_ids) = match systemtray::init_tray(&tunnels, &tunnel_manager) {
            Ok((icon, ids)) => (Some(icon), Some(ids)),
            Err(e) => {
                log_print(&format!("Error initializing system tray: {}", e));
                std::process::exit(1);
            }
        };

        log_print("Drill initialized. Application running...");

        (
            Self {
                windows: BTreeMap::new(),
                tunnel_manager,
                tunnels_file,
                tray_icon,
                menu_ids,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TrayMenuEvent(event) => {
                log_print(&format!("Received tray menu event: {:?}", event.id));
                
                if let Some(menu_ids) = &self.menu_ids {
                    // Check for Create action
                    if event.id == menu_ids.create {
                        return self.update(Message::OpenCreateTunnel);
                    }

                    // Check for About action
                    if event.id == menu_ids.about {
                        return self.update(Message::OpenAbout);
                    }

                    // Check for Quit action
                    if event.id == menu_ids.quit {
                        return self.update(Message::Quit);
                    }

                    // Check for tunnel connect events
                    for (tunnel_name, connect_id) in &menu_ids.tunnel_connect {
                        if event.id == *connect_id {
                            return self.update(Message::TunnelConnect(tunnel_name.clone()));
                        }
                    }

                    // Check for tunnel disconnect events
                    for (tunnel_name, disconnect_id) in &menu_ids.tunnel_disconnect {
                        if event.id == *disconnect_id {
                            return self.update(Message::TunnelDisconnect(tunnel_name.clone()));
                        }
                    }

                    // Check for tunnel edit events
                    for (tunnel_name, edit_id) in &menu_ids.tunnel_edit {
                        if event.id == *edit_id {
                            return self.update(Message::TunnelEdit(tunnel_name.clone()));
                        }
                    }

                    // Check for tunnel remove events
                    for (tunnel_name, remove_id) in &menu_ids.tunnel_remove {
                        if event.id == *remove_id {
                            return self.update(Message::TunnelRemove(tunnel_name.clone()));
                        }
                    }
                }
                Task::none()
            }

            Message::OpenAbout => {
                // Check if About window is already open
                if let Some((window_id, _)) = self.windows.iter().find(|(_, wt)| matches!(wt, WindowType::About)) {
                    log_print("About window already open, bringing to front...");
                    return window::gain_focus(*window_id);
                }

                log_print("Opening About window...");
                let (id, open) = window::open(window::Settings {
                    size: Size::new(400.0, 300.0),
                    resizable: false,
                    ..window::Settings::default()
                });

                open.then(move |_| Task::done(Message::WindowOpened(id, WindowType::About)))
            }

            Message::OpenCreateTunnel => {
                // Check if CreateTunnel window is already open
                if let Some((window_id, _)) = self.windows.iter().find(|(_, wt)| matches!(wt, WindowType::CreateTunnel { .. })) {
                    log_print("Create Tunnel window already open, bringing to front...");
                    return window::gain_focus(*window_id);
                }

                log_print("Opening Create Tunnel window...");
                let (id, open) = window::open(window::Settings {
                    size: Size::new(500.0, 630.0),
                    resizable: false,
                    ..window::Settings::default()
                });

                open.then(move |_| {
                    Task::done(Message::WindowOpened(
                        id,
                        WindowType::new_create_tunnel(),
                    ))
                })
            }

            Message::TunnelConnect(tunnel_name) => {
                log_print(&format!("Connect tunnel '{}'", tunnel_name));
                let manager = self.tunnel_manager.lock().unwrap();
                if let Some(tunnel) = manager.get_tunnels().iter().find(|t| t.name == tunnel_name)
                {
                    match manager.start_tunnel(tunnel) {
                        Ok(_) => {
                            notifications::notify_tunnel_connected(&tunnel_name);
                        }
                        Err(e) => {
                            log_print(&format!(
                                "Error starting tunnel '{}': {}",
                                tunnel_name, e
                            ));
                            notifications::notify_tunnel_error(&tunnel_name, &e.to_string());
                        }
                    }
                }
                drop(manager);
                self.update(Message::UpdateTrayMenu)
            }

            Message::TunnelDisconnect(tunnel_name) => {
                log_print(&format!("Disconnect tunnel '{}'", tunnel_name));
                let manager = self.tunnel_manager.lock().unwrap();
                match manager.stop_tunnel(&tunnel_name) {
                    Ok(_) => {
                        notifications::notify_tunnel_disconnected(&tunnel_name);
                    }
                    Err(e) => {
                        log_print(&format!(
                            "Error stopping tunnel '{}': {}",
                            tunnel_name, e
                        ));
                    }
                }
                drop(manager);
                self.update(Message::UpdateTrayMenu)
            }

            Message::TunnelEdit(tunnel_name) => {
                log_print(&format!("Edit tunnel '{}'", tunnel_name));
                
                // Check if Edit window is already open for this tunnel
                if let Some((window_id, _)) = self.windows.iter().find(|(_, wt)| {
                    matches!(wt, WindowType::EditTunnel { name, .. } if name == &tunnel_name)
                }) {
                    log_print("Edit window already open for this tunnel, bringing to front...");
                    return window::gain_focus(*window_id);
                }

                // Find the tunnel and open edit window
                let manager = self.tunnel_manager.lock().unwrap();
                if let Some(tunnel) = manager.get_tunnels().iter().find(|t| t.name == tunnel_name) {
                    let tunnel_clone = tunnel.clone();
                    drop(manager);

                    log_print("Opening Edit Tunnel window...");
                    let (id, open) = window::open(window::Settings {
                        size: Size::new(600.0, 700.0),
                        resizable: true,
                        ..window::Settings::default()
                    });

                    return open.then(move |_| {
                        Task::done(Message::WindowOpened(
                            id,
                            WindowType::new_edit_tunnel(&tunnel_clone),
                        ))
                    });
                } else {
                    log_print(&format!("Tunnel '{}' not found", tunnel_name));
                    drop(manager);
                }
                Task::none()
            }

            Message::TunnelRemove(tunnel_name) => {
                log_print(&format!("Remove tunnel '{}'", tunnel_name));
                let mut manager = self.tunnel_manager.lock().unwrap();
                match manager.remove_tunnel(&tunnel_name) {
                    Ok(_) => {
                        // Save the updated tunnels list
                        if let Err(e) =
                            TunnelManager::save_tunnels(&self.tunnels_file, manager.get_tunnels())
                        {
                            log_print(&format!("Error saving tunnels: {}", e));
                        } else {
                            notifications::notify_tunnel_removed(&tunnel_name);
                        }
                    }
                    Err(e) => {
                        log_print(&format!(
                            "Error removing tunnel '{}': {}",
                            tunnel_name, e
                        ));
                    }
                }
                drop(manager);
                self.update(Message::UpdateTrayMenu)
            }

            Message::Quit => {
                log_print("Quit selected from tray menu");
                let manager = self.tunnel_manager.lock().unwrap();
                manager.cleanup();
                drop(manager);
                iced::exit()
            }

            Message::WindowOpened(id, window_type) => {
                self.windows.insert(id, window_type);
                Task::none()
            }

            Message::WindowClosed(id) => {
                self.windows.remove(&id);
                Task::none()
            }

            // Message::AboutClose => {
            //     // Find the About window and close it
            //     let window_id = self
            //         .windows
            //         .iter()
            //         .find_map(|(id, win_type)| match win_type {
            //             WindowType::About => Some(*id),
            //             _ => None,
            //         });

            //     if let Some(id) = window_id {
            //         window::close(id)
            //     } else {
            //         Task::none()
            //     }
            // }

            Message::CreateTunnelNameChanged(value) => {
                self.update_create_tunnel_field(|ct| {
                    if let WindowType::CreateTunnel { name, .. } = ct {
                        *name = value.clone();
                    }
                });
                Task::none()
            }

            Message::CreateTunnelLocalHostChanged(value) => {
                self.update_create_tunnel_field(|ct| {
                    if let WindowType::CreateTunnel { local_host, .. } = ct {
                        *local_host = value.clone();
                    }
                });
                Task::none()
            }

            Message::CreateTunnelLocalPortChanged(value) => {
                self.update_create_tunnel_field(|ct| {
                    if let WindowType::CreateTunnel { local_port, .. } = ct {
                        *local_port = value.clone();
                    }
                });
                Task::none()
            }

            Message::CreateTunnelRemoteHostChanged(value) => {
                self.update_create_tunnel_field(|ct| {
                    if let WindowType::CreateTunnel { remote_host, .. } = ct {
                        *remote_host = value.clone();
                    }
                });
                Task::none()
            }

            Message::CreateTunnelRemotePortChanged(value) => {
                self.update_create_tunnel_field(|ct| {
                    if let WindowType::CreateTunnel { remote_port, .. } = ct {
                        *remote_port = value.clone();
                    }
                });
                Task::none()
            }

            Message::CreateTunnelSshUserChanged(value) => {
                self.update_create_tunnel_field(|ct| {
                    if let WindowType::CreateTunnel { ssh_user, .. } = ct {
                        *ssh_user = value.clone();
                    }
                });
                Task::none()
            }

            Message::CreateTunnelSshHostChanged(value) => {
                self.update_create_tunnel_field(|ct| {
                    if let WindowType::CreateTunnel { ssh_host, .. } = ct {
                        *ssh_host = value.clone();
                    }
                });
                Task::none()
            }

            Message::CreateTunnelSshPortChanged(value) => {
                self.update_create_tunnel_field(|ct| {
                    if let WindowType::CreateTunnel { ssh_port, .. } = ct {
                        *ssh_port = value.clone();
                    }
                });
                Task::none()
            }

            Message::CreateTunnelPrivateKeyChanged(value) => {
                self.update_create_tunnel_field(|ct| {
                    if let WindowType::CreateTunnel { private_key, .. } = ct {
                        *private_key = value.clone();
                    }
                });
                Task::none()
            }

            Message::CreateTunnelBrowsePrivateKey => {
                if let Some(path) = windows::create_tunnel::browse_for_private_key() {
                    self.update_create_tunnel_field(|ct| {
                        if let WindowType::CreateTunnel { private_key, .. } = ct {
                            *private_key = path.clone();
                        }
                    });
                }
                Task::none()
            }

            Message::CreateTunnelTest(window_id) => {
                // Get the window data and test the connection
                if let Some(WindowType::CreateTunnel {
                    name,
                    local_host,
                    local_port,
                    remote_host,
                    remote_port,
                    ssh_user,
                    ssh_host,
                    ssh_port,
                    private_key,
                    error_message,
                    test_message,
                }) = self.windows.get_mut(&window_id)
                {
                    // Clear previous messages
                    *error_message = None;
                    *test_message = None;

                    // Validate basic fields before testing
                    match windows::create_tunnel::validate_and_create_tunnel(
                        name,
                        local_host,
                        local_port,
                        remote_host,
                        remote_port,
                        ssh_user,
                        ssh_host,
                        ssh_port,
                        private_key,
                    ) {
                        Ok(tunnel) => {
                            // Test the SSH connection
                            match TunnelManager::test_tunnel(&tunnel) {
                                Ok(success_msg) => {
                                    *test_message = Some(success_msg);
                                }
                                Err(err_msg) => {
                                    *test_message = Some(err_msg);
                                }
                            }
                            // Resize window to accommodate message
                            // Calculate extra height based on message length
                            let extra_height = test_message.as_ref().map(|msg| {
                                let lines = (msg.len() / 60).max(1) as f32;
                                lines * 20.0 + 40.0
                            }).unwrap_or(0.0);
                            return window::resize(window_id, Size::new(500.0, 640.0 + extra_height));
                        }
                        Err(err) => {
                            *error_message = Some(err);
                            // Resize window to accommodate error message
                            let extra_height = error_message.as_ref().map(|msg| {
                                let lines = (msg.len() / 60).max(1) as f32;
                                lines * 20.0 + 40.0
                            }).unwrap_or(0.0);
                            return window::resize(window_id, Size::new(500.0, 640.0 + extra_height));
                        }
                    }
                }
                Task::none()
            }

            Message::CreateTunnelCreate(window_id) => {
                // Get the window data
                if let Some(WindowType::CreateTunnel {
                    name,
                    local_host,
                    local_port,
                    remote_host,
                    remote_port,
                    ssh_user,
                    ssh_host,
                    ssh_port,
                    private_key,
                    error_message,
                    test_message: _,
                }) = self.windows.get_mut(&window_id)
                {
                    match windows::create_tunnel::validate_and_create_tunnel(
                        name,
                        local_host,
                        local_port,
                        remote_host,
                        remote_port,
                        ssh_user,
                        ssh_host,
                        ssh_port,
                        private_key,
                    ) {
                        Ok(tunnel) => {
                            log_print(&format!("Saving new tunnel: {}", tunnel.name));

                            // Add tunnel to manager
                            let mut manager = self.tunnel_manager.lock().unwrap();
                            manager.add_tunnel(tunnel.clone());

                            // Save to file
                            if let Err(e) = TunnelManager::save_tunnels(
                                &self.tunnels_file,
                                manager.get_tunnels(),
                            ) {
                                log_print(&format!("Error saving tunnels: {}", e));
                            } else {
                                notifications::notify_tunnel_created(&tunnel.name);
                            }
                            drop(manager);

                            // Update tray menu and close window
                            return Task::batch(vec![
                                self.update(Message::UpdateTrayMenu),
                                window::close(window_id),
                            ]);
                        }
                        Err(err) => {
                            *error_message = Some(err);
                            // Resize window to accommodate error message
                            let extra_height = error_message.as_ref().map(|msg| {
                                let lines = (msg.len() / 60).max(1) as f32;
                                lines * 20.0 + 40.0
                            }).unwrap_or(0.0);
                            return window::resize(window_id, Size::new(500.0, 640.0 + extra_height));
                        }
                    }
                }
                Task::none()
            }

            Message::CreateTunnelCancel(window_id) => window::close(window_id),

            // Edit Tunnel message handlers
            Message::EditTunnelNameChanged(value) => {
                self.update_edit_tunnel_field(|et| {
                    if let WindowType::EditTunnel { name, .. } = et {
                        *name = value.clone();
                    }
                });
                Task::none()
            }

            Message::EditTunnelLocalHostChanged(value) => {
                self.update_edit_tunnel_field(|et| {
                    if let WindowType::EditTunnel { local_host, .. } = et {
                        *local_host = value.clone();
                    }
                });
                Task::none()
            }

            Message::EditTunnelLocalPortChanged(value) => {
                self.update_edit_tunnel_field(|et| {
                    if let WindowType::EditTunnel { local_port, .. } = et {
                        *local_port = value.clone();
                    }
                });
                Task::none()
            }

            Message::EditTunnelRemoteHostChanged(value) => {
                self.update_edit_tunnel_field(|et| {
                    if let WindowType::EditTunnel { remote_host, .. } = et {
                        *remote_host = value.clone();
                    }
                });
                Task::none()
            }

            Message::EditTunnelRemotePortChanged(value) => {
                self.update_edit_tunnel_field(|et| {
                    if let WindowType::EditTunnel { remote_port, .. } = et {
                        *remote_port = value.clone();
                    }
                });
                Task::none()
            }

            Message::EditTunnelSshUserChanged(value) => {
                self.update_edit_tunnel_field(|et| {
                    if let WindowType::EditTunnel { ssh_user, .. } = et {
                        *ssh_user = value.clone();
                    }
                });
                Task::none()
            }

            Message::EditTunnelSshHostChanged(value) => {
                self.update_edit_tunnel_field(|et| {
                    if let WindowType::EditTunnel { ssh_host, .. } = et {
                        *ssh_host = value.clone();
                    }
                });
                Task::none()
            }

            Message::EditTunnelSshPortChanged(value) => {
                self.update_edit_tunnel_field(|et| {
                    if let WindowType::EditTunnel { ssh_port, .. } = et {
                        *ssh_port = value.clone();
                    }
                });
                Task::none()
            }

            Message::EditTunnelPrivateKeyChanged(value) => {
                self.update_edit_tunnel_field(|et| {
                    if let WindowType::EditTunnel { private_key, .. } = et {
                        *private_key = value.clone();
                    }
                });
                Task::none()
            }

            Message::EditTunnelBrowsePrivateKey => {
                if let Some(path) = windows::create_tunnel::browse_for_private_key() {
                    self.update_edit_tunnel_field(|et| {
                        if let WindowType::EditTunnel { private_key, .. } = et {
                            *private_key = path.clone();
                        }
                    });
                }
                Task::none()
            }

            Message::EditTunnelTest(window_id) => {
                // Get the window data and test the connection
                if let Some(WindowType::EditTunnel {
                    tunnel_id: _,
                    name,
                    local_host,
                    local_port,
                    remote_host,
                    remote_port,
                    ssh_user,
                    ssh_host,
                    ssh_port,
                    private_key,
                    error_message,
                    test_message,
                }) = self.windows.get_mut(&window_id)
                {
                    // Clear previous messages
                    *error_message = None;
                    *test_message = None;

                    let test_tunnel = crate::tunnels::Tunnel {
                        id: String::new(), // Not needed for testing
                        name: name.clone(),
                        local_host: local_host.clone(),
                        local_port: local_port.clone(),
                        remote_host: remote_host.clone(),
                        remote_port: remote_port.clone(),
                        ssh_user: ssh_user.clone(),
                        ssh_host: ssh_host.clone(),
                        ssh_port: ssh_port.clone(),
                        private_key: private_key.clone(),
                    };

                    match TunnelManager::test_tunnel(&test_tunnel) {
                        Ok(msg) => {
                            *test_message = Some(msg);
                        }
                        Err(err) => {
                            *test_message = Some(err);
                        }
                    }
                }
                Task::none()
            }

            Message::EditTunnelSave(window_id) => {
                // Get the window data
                if let Some(WindowType::EditTunnel {
                    tunnel_id,
                    name,
                    local_host,
                    local_port,
                    remote_host,
                    remote_port,
                    ssh_user,
                    ssh_host,
                    ssh_port,
                    private_key,
                    error_message,
                    test_message: _,
                }) = self.windows.get_mut(&window_id)
                {
                    match windows::create_tunnel::validate_and_create_tunnel(
                        name,
                        local_host,
                        local_port,
                        remote_host,
                        remote_port,
                        ssh_user,
                        ssh_host,
                        ssh_port,
                        private_key,
                    ) {
                        Ok(mut tunnel) => {
                            log_print(&format!("Updating tunnel: {}", tunnel.name));

                            // Keep the original tunnel ID
                            tunnel.id = tunnel_id.clone();

                            // Update tunnel in manager
                            let mut manager = self.tunnel_manager.lock().unwrap();
                            if let Err(e) = manager.update_tunnel(tunnel_id, tunnel.clone()) {
                                log_print(&format!("Error updating tunnel: {}", e));
                                *error_message = Some(format!("Error updating tunnel: {}", e));
                                drop(manager);
                                return Task::none();
                            }

                            // Save to file
                            if let Err(e) = TunnelManager::save_tunnels(
                                &self.tunnels_file,
                                manager.get_tunnels(),
                            ) {
                                log_print(&format!("Error saving tunnels: {}", e));
                            }
                            drop(manager);

                            // Update tray menu and close window
                            return Task::batch(vec![
                                self.update(Message::UpdateTrayMenu),
                                window::close(window_id),
                            ]);
                        }
                        Err(err) => {
                            *error_message = Some(err);
                            // Resize window to accommodate error message
                            let extra_height = error_message.as_ref().map(|msg| {
                                let lines = (msg.len() / 60).max(1) as f32;
                                lines * 20.0 + 40.0
                            }).unwrap_or(0.0);
                            return window::resize(window_id, Size::new(500.0, 640.0 + extra_height));
                        }
                    }
                }
                Task::none()
            }

            Message::EditTunnelCancel(window_id) => window::close(window_id),

            Message::UpdateTrayMenu => {
                if let (Some(tray_icon), Some(_)) = (&mut self.tray_icon, &self.menu_ids) {
                    let manager = self.tunnel_manager.lock().unwrap();
                    let tunnels = manager.get_tunnels().clone();
                    drop(manager);

                    match systemtray::update_tray_menu(tray_icon, &tunnels, &self.tunnel_manager)
                    {
                        Ok(new_ids) => {
                            self.menu_ids = Some(new_ids);
                        }
                        Err(e) => {
                            log_print(&format!("Error updating tray menu: {}", e));
                        }
                    }
                }
                Task::none()
            }
        }
    }

    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if let Some(window_type) = self.windows.get(&window_id) {
            match window_type {
                WindowType::About => {
                    windows::about::view().map(|msg| match msg {
                        // windows::about::Message::Close => Message::AboutClose,
                    })
                }
                WindowType::CreateTunnel {
                    name,
                    local_host,
                    local_port,
                    remote_host,
                    remote_port,
                    ssh_user,
                    ssh_host,
                    ssh_port,
                    private_key,
                    error_message,
                    test_message,
                } => windows::create_tunnel::view(
                    false, // is_edit_mode
                    name,
                    local_host,
                    local_port,
                    remote_host,
                    remote_port,
                    ssh_user,
                    ssh_host,
                    ssh_port,
                    private_key,
                    error_message,
                    test_message,
                )
                .map(move |msg| match msg {
                    windows::create_tunnel::Message::NameChanged(v) => {
                        Message::CreateTunnelNameChanged(v)
                    }
                    windows::create_tunnel::Message::LocalHostChanged(v) => {
                        Message::CreateTunnelLocalHostChanged(v)
                    }
                    windows::create_tunnel::Message::LocalPortChanged(v) => {
                        Message::CreateTunnelLocalPortChanged(v)
                    }
                    windows::create_tunnel::Message::RemoteHostChanged(v) => {
                        Message::CreateTunnelRemoteHostChanged(v)
                    }
                    windows::create_tunnel::Message::RemotePortChanged(v) => {
                        Message::CreateTunnelRemotePortChanged(v)
                    }
                    windows::create_tunnel::Message::SshUserChanged(v) => {
                        Message::CreateTunnelSshUserChanged(v)
                    }
                    windows::create_tunnel::Message::SshHostChanged(v) => {
                        Message::CreateTunnelSshHostChanged(v)
                    }
                    windows::create_tunnel::Message::SshPortChanged(v) => {
                        Message::CreateTunnelSshPortChanged(v)
                    }
                    windows::create_tunnel::Message::PrivateKeyChanged(v) => {
                        Message::CreateTunnelPrivateKeyChanged(v)
                    }
                    windows::create_tunnel::Message::BrowsePrivateKey => {
                        Message::CreateTunnelBrowsePrivateKey
                    }
                    windows::create_tunnel::Message::Test => {
                        Message::CreateTunnelTest(window_id)
                    }
                    windows::create_tunnel::Message::Create => {
                        Message::CreateTunnelCreate(window_id)
                    }
                    windows::create_tunnel::Message::Cancel => {
                        Message::CreateTunnelCancel(window_id)
                    }
                }),
                WindowType::EditTunnel {
                    tunnel_id: _,
                    name,
                    local_host,
                    local_port,
                    remote_host,
                    remote_port,
                    ssh_user,
                    ssh_host,
                    ssh_port,
                    private_key,
                    error_message,
                    test_message,
                } => windows::create_tunnel::view(
                    true, // is_edit_mode
                    name,
                    local_host,
                    local_port,
                    remote_host,
                    remote_port,
                    ssh_user,
                    ssh_host,
                    ssh_port,
                    private_key,
                    error_message,
                    test_message,
                )
                .map(move |msg| match msg {
                    windows::create_tunnel::Message::NameChanged(v) => {
                        Message::EditTunnelNameChanged(v)
                    }
                    windows::create_tunnel::Message::LocalHostChanged(v) => {
                        Message::EditTunnelLocalHostChanged(v)
                    }
                    windows::create_tunnel::Message::LocalPortChanged(v) => {
                        Message::EditTunnelLocalPortChanged(v)
                    }
                    windows::create_tunnel::Message::RemoteHostChanged(v) => {
                        Message::EditTunnelRemoteHostChanged(v)
                    }
                    windows::create_tunnel::Message::RemotePortChanged(v) => {
                        Message::EditTunnelRemotePortChanged(v)
                    }
                    windows::create_tunnel::Message::SshUserChanged(v) => {
                        Message::EditTunnelSshUserChanged(v)
                    }
                    windows::create_tunnel::Message::SshHostChanged(v) => {
                        Message::EditTunnelSshHostChanged(v)
                    }
                    windows::create_tunnel::Message::SshPortChanged(v) => {
                        Message::EditTunnelSshPortChanged(v)
                    }
                    windows::create_tunnel::Message::PrivateKeyChanged(v) => {
                        Message::EditTunnelPrivateKeyChanged(v)
                    }
                    windows::create_tunnel::Message::BrowsePrivateKey => {
                        Message::EditTunnelBrowsePrivateKey
                    }
                    windows::create_tunnel::Message::Test => {
                        Message::EditTunnelTest(window_id)
                    }
                    windows::create_tunnel::Message::Create => {
                        Message::EditTunnelSave(window_id)
                    }
                    windows::create_tunnel::Message::Cancel => {
                        Message::EditTunnelCancel(window_id)
                    }
                }),
            }
        } else {
            // No window found - this shouldn't happen
            iced::widget::text("Window not found").into()
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // Listen for window close events
        let window_events =
            iced::event::listen_with(|event, _status, id| match event {
                iced::Event::Window(window_event) => match window_event {
                    iced::window::Event::Closed => Some(Message::WindowClosed(id)),
                    _ => None,
                },
                _ => None,
            });

        // Poll tray menu events periodically
        struct TrayEventsPoll;
        
        let tray_subscription = Subscription::run_with_id(
            std::any::TypeId::of::<TrayEventsPoll>(),
            iced::stream::channel(100, |mut output| async move {
                loop {
                    // Check for menu events
                    let menu_channel = MenuEvent::receiver();
                    while let Ok(event) = menu_channel.try_recv() {
                        let _ = output.send(event).await;
                    }
                    
                    // Small delay to avoid busy-waiting  
                    tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
                }
            })
        ).map(Message::TrayMenuEvent);

        Subscription::batch(vec![window_events, tray_subscription])
    }

    // Helper methods for iced::daemon function references
    pub fn title_fn(_app: &App, _id: window::Id) -> String {
        String::from("Drill")
    }

    pub fn update_fn(app: &mut App, message: Message) -> Task<Message> {
        app.update(message)
    }

    pub fn view_fn<'a>(app: &'a App, id: window::Id) -> Element<'a, Message> {
        app.view(id)
    }

    pub fn subscription_fn(app: &App) -> Subscription<Message> {
        app.subscription()
    }

    fn update_create_tunnel_field<F>(&mut self, updater: F)
    where
        F: Fn(&mut WindowType),
    {
        for window_type in self.windows.values_mut() {
            if matches!(window_type, WindowType::CreateTunnel { .. }) {
                updater(window_type);
                break;
            }
        }
    }

    fn update_edit_tunnel_field<F>(&mut self, updater: F)
    where
        F: Fn(&mut WindowType),
    {
        for window_type in self.windows.values_mut() {
            if matches!(window_type, WindowType::EditTunnel { .. }) {
                updater(window_type);
                break;
            }
        }
    }
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
