use crate::config;
use crate::logs::log_print;
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
    CreateTunnelCreate(window::Id),
    CreateTunnelCancel(window::Id),

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
                    size: Size::new(500.0, 640.0),
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
                    if let Err(e) = manager.start_tunnel(tunnel) {
                        log_print(&format!(
                            "Error starting tunnel '{}': {}",
                            tunnel_name, e
                        ));
                    }
                }
                drop(manager);
                self.update(Message::UpdateTrayMenu)
            }

            Message::TunnelDisconnect(tunnel_name) => {
                log_print(&format!("Disconnect tunnel '{}'", tunnel_name));
                let manager = self.tunnel_manager.lock().unwrap();
                if let Err(e) = manager.stop_tunnel(&tunnel_name) {
                    log_print(&format!(
                        "Error stopping tunnel '{}': {}",
                        tunnel_name, e
                    ));
                }
                drop(manager);
                self.update(Message::UpdateTrayMenu)
            }

            Message::TunnelRemove(tunnel_name) => {
                log_print(&format!("Remove tunnel '{}'", tunnel_name));
                let mut manager = self.tunnel_manager.lock().unwrap();
                if let Err(e) = manager.remove_tunnel(&tunnel_name) {
                    log_print(&format!(
                        "Error removing tunnel '{}': {}",
                        tunnel_name, e
                    ));
                } else {
                    // Save the updated tunnels list
                    if let Err(e) =
                        TunnelManager::save_tunnels(&self.tunnels_file, manager.get_tunnels())
                    {
                        log_print(&format!("Error saving tunnels: {}", e));
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
                    error_message,
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
                        }
                    }
                }
                Task::none()
            }

            Message::CreateTunnelCancel(window_id) => window::close(window_id),

            Message::UpdateTrayMenu => {
                if let (Some(tray_icon), Some(_)) = (&mut self.tray_icon, &self.menu_ids) {
                    let manager = self.tunnel_manager.lock().unwrap();
                    let tunnels = manager.get_tunnels().clone();
                    drop(manager);

                    match systemtray::update_tray_menu(tray_icon, &tunnels, &self.tunnel_manager)
                    {
                        Ok(new_ids) => {
                            self.menu_ids = Some(new_ids);
                            log_print("Tray menu updated");
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

    pub fn view(&self, window_id: window::Id) -> Element<Message> {
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
                    error_message,
                } => windows::create_tunnel::view(
                    name,
                    local_host,
                    local_port,
                    remote_host,
                    remote_port,
                    ssh_user,
                    ssh_host,
                    ssh_port,
                    error_message,
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
                    windows::create_tunnel::Message::Create => {
                        Message::CreateTunnelCreate(window_id)
                    }
                    windows::create_tunnel::Message::Cancel => {
                        Message::CreateTunnelCancel(window_id)
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
