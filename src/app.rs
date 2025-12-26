use crate::config;
use crate::logs::Logger;
use crate::notifications;
use crate::systemtray::{self, TrayMenuIds};
use crate::tunnels::{TunnelManager, StatusUpdate};
use tokio::sync::broadcast;
use crate::windows::{self, WindowType};
use iced::futures::SinkExt;
use iced::window;
use iced::{Element, Size, Subscription, Task};
use std::collections::BTreeMap;
use std::path::PathBuf;
// use std::sync::{Mutex};
use tray_icon::menu::MenuEvent;
use tray_icon::TrayIcon;



pub struct App {
    windows: BTreeMap<window::Id, WindowType>,
    tunnel_manager: TunnelManager,
    tunnels_file: PathBuf,
    tray_icon: Option<TrayIcon>,
    menu_ids: Option<TrayMenuIds>,
    logger: Logger,
    status_receiver: broadcast::Receiver<StatusUpdate>,
}

/// Identifies which field in the tunnel form was changed
#[derive(Debug, Clone)]
pub enum TunnelFormField {
    Name(String),
    LocalHost(String),
    LocalPort(String),
    RemoteHost(String),
    RemotePort(String),
    SshUser(String),
    SshHost(String),
    SshPort(String),
    PrivateKey(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    // Tray menu events
    TrayMenuEvent(MenuEvent),
    OpenAbout,
    OpenCreateTunnel,
    TunnelConnect(String),
    TunnelDisconnect(String),
    TunnelOpenWeb(String),
    TunnelEdit(String),
    TunnelRemove(String),
    Quit,

    // Tunnel status monitoring
    TunnelStatusUpdate(StatusUpdate),

    // Window events
    WindowOpened(window::Id, WindowType),
    WindowClosed(window::Id),

    // Unified tunnel form messages (handles both create and edit)
    TunnelFormFieldChanged(window::Id, TunnelFormField),
    TunnelFormBrowsePrivateKey(window::Id),
    TunnelFormTest(window::Id),
    TunnelFormSubmit(window::Id),
    TunnelFormCancel(window::Id),

    // Internal
    UpdateTrayMenu,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let (config_path, mut logger): (std::path::PathBuf, crate::logs::Logger) = match config::init_config() {
            Ok((config_path, logger)) => (config_path, logger),
            Err(e) => {
                println!("Error initializing configuration: {}", e);
                std::process::exit(1);
            }
        };
        logger.log_print("Drill - Multi-Platform tunnel drilling Application");
        logger.log_print(&format!("Platform: {}", get_platform()));
        logger.log_print("");
        logger.log_print(&format!("Configuration loaded from: {}", config_path.display()));

        // Load tunnels from the tunnels file
        let tunnels_file = match config::get_tunnels_file_path() {
            Ok(path) => path,
            Err(e) => {
                logger.log_print(&format!("Error getting tunnels file path: {}", e));
                std::process::exit(1);
            }
        };

        let tunnels = match TunnelManager::load_tunnels(&tunnels_file) {
            Ok(t) => t,
            Err(e) => {
                logger.log_print(&format!("Error loading tunnels: {}", e));
                Vec::new()
            }
        };

        // Create tunnel manager
        let mut tunnel_manager = TunnelManager::new();
        tunnel_manager.set_tunnels(tunnels.clone());
        // Create status channel
        let (status_tx, status_rx) = broadcast::channel(100);
        tunnel_manager.set_status_channel(status_tx);
        let status_receiver = status_rx;

        // Initialize system tray
        let tunnel_statuses: Vec<(String, crate::tunnels::TunnelStatus)> = tunnel_manager
            .get_tunnels()
            .iter()
            .map(|t| (t.name.clone(), tunnel_manager.get_tunnel_status(&t.name)))
            .collect();
        let (tray_icon, menu_ids) = match systemtray::init_tray(&tunnels, &tunnel_statuses) {
            Ok((icon, ids)) => (Some(icon), Some(ids)),
            Err(e) => {
                logger.log_print(&format!("Error initializing system tray: {}", e));
                std::process::exit(1);
            }
        };

        logger.log_print("Drill initialized. Application running...");

        (
            Self {
                windows: BTreeMap::new(),
                tunnel_manager,
                tunnels_file,
                tray_icon,
                menu_ids,
                logger,
                status_receiver,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TrayMenuEvent(event) => {
                self.handle_tray_menu_event(event)
            }


            Message::OpenAbout => {
                // Check if About window is already open
                if let Some((window_id, _)) = self.windows.iter().find(|(_, wt)| matches!(wt, WindowType::About)) {
                    // log_print("About window already open, bringing to front...");
                    return window::gain_focus(*window_id);
                }
                // log_print("Opening About window...");
                let (id, open) = window::open(window::Settings {
                    size: Size::new(400.0, 300.0),
                    resizable: false,
                    ..window::Settings::default()
                });
                open.then(move |_| Task::done(Message::WindowOpened(id, WindowType::About)))
            }

            Message::OpenCreateTunnel => {
                // Check if TunnelForm window in Create mode is already open
                if let Some((window_id, _wt)) = self.windows.iter().find(|(_, wt)| {
                    matches!(wt, WindowType::TunnelForm { mode: windows::FormMode::Create, .. })
                }) {
                    // log_print("Create Tunnel window already open, bringing to front...");
                    return window::gain_focus(*window_id);
                }
                // log_print("Opening Create Tunnel window...");
                let (id, open) = window::open(window::Settings {
                    size: Size::new(500.0, 630.0),
                    resizable: false,
                    ..window::Settings::default()
                });
                open.then(move |_| {
                    Task::done(Message::WindowOpened(
                        id,
                        WindowType::new_tunnel_form_create(),
                    ))
                })
            }

            Message::TunnelStatusUpdate(update) => {
                match update {
                    StatusUpdate::Connecting(_tunnel_name) => {
                        // log_print(&format!("Tunnel '{}' is connecting...", _tunnel_name));
                    }
                    StatusUpdate::Connected(tunnel_name) => {
                        // log_print(&format!("Tunnel '{}' connected successfully", tunnel_name));
                        let _ = notifications::notify_tunnel_connected(&tunnel_name);
                        return self.update(Message::UpdateTrayMenu);
                    }
                    StatusUpdate::Error(tunnel_name, error) => {
                        // log_print(&format!("Tunnel '{}' error: {}", tunnel_name, error));
                        notifications::notify_tunnel_error(&tunnel_name, &error.to_string());
                        return self.update(Message::UpdateTrayMenu);
                    }
                    StatusUpdate::Disconnected(_tunnel_name) => {
                        // log_print(&format!("Tunnel '{}' disconnected", _tunnel_name));
                        return self.update(Message::UpdateTrayMenu);
                    }
                }
                Task::none()
            }

            Message::TunnelConnect(tunnel_name) => {
                // log_print(&format!("Connect tunnel '{}'", tunnel_name));
                if let Some(tunnel) = self.tunnel_manager.get_tunnels().iter().find(|t| t.name == tunnel_name).cloned() {
                    match self.tunnel_manager.start_tunnel(&tunnel) {
                        Ok(_) => {
                            let _ = notifications::notify_tunnel_connected(&tunnel_name);
                        }
                        Err(e) => {
                            self.logger.log_print(&format!(
                                "Error starting tunnel '{}': {}",
                                tunnel_name, e
                            ));
                            notifications::notify_tunnel_error(&tunnel_name, &e.to_string());
                        }
                    }
                }
                self.update(Message::UpdateTrayMenu)
            }

            Message::TunnelDisconnect(tunnel_name) => {
                // log_print(&format!("Disconnect tunnel '{}'", tunnel_name));
                match self.tunnel_manager.stop_tunnel(&tunnel_name) {
                    Ok(_) => {
                        notifications::notify_tunnel_disconnected(&tunnel_name);
                    }
                    Err(e) => {
                        self.logger.log_print(&format!(
                            "Error stopping tunnel '{}': {}",
                            tunnel_name, e
                        ));
                    }
                }
                self.update(Message::UpdateTrayMenu)
            }

            Message::TunnelOpenWeb(tunnel_name) => {
                // log_print(&format!("Open web for tunnel '{}'", tunnel_name));
                if let Some(tunnel) = self.tunnel_manager.get_tunnels().iter().find(|t| t.name == tunnel_name) {
                    let url = format!("http://{}:{}", tunnel.local_host, tunnel.local_port);
                    self.logger.log_print(&format!("Opening URL: {}", url));
                        if let Err(_e) =
                            TunnelManager::save_tunnels(&self.tunnels_file, self.tunnel_manager.get_tunnels())
                        {
                            self.logger.log_print(&format!("Error saving tunnels: {}", _e));
                    }
                } else {
                    self.logger.log_print(&format!("Tunnel '{}' not found", tunnel_name));
                }
                Task::none()
            }

            Message::TunnelEdit(tunnel_name) => {
                // log_print(&format!("Edit tunnel '{}'", tunnel_name));
                // Check if TunnelForm window in Edit mode for this tunnel is already open
                if let Some((window_id, _wt)) = self.windows.iter().find(|(_, wt)| {
                    matches!(wt, WindowType::TunnelForm { mode: windows::FormMode::Edit { tunnel_id }, .. } if {
                        let manager = &self.tunnel_manager;
                        manager.get_tunnels().iter().any(|t| t.name == tunnel_name && &t.id == tunnel_id)
                    })
                }) {
                    // log_print("Edit window already open for this tunnel, bringing to front...");
                    return window::gain_focus(*window_id);
                }
                // Find the tunnel and open edit window
                if let Some(tunnel) = self.tunnel_manager.get_tunnels().iter().find(|t| t.name == tunnel_name) {
                    let tunnel_clone = tunnel.clone();
                    self.logger.log_print("Opening Edit Tunnel window...");
                    let (id, open) = window::open(window::Settings {
                        size: Size::new(600.0, 700.0),
                        resizable: true,
                        ..window::Settings::default()
                    });
                    return open.then(move |_| {
                        Task::done(Message::WindowOpened(
                            id,
                            WindowType::new_tunnel_form_edit(&tunnel_clone),
                        ))
                    });
                } else {
                    self.logger.log_print(&format!("Tunnel '{}' not found", tunnel_name));
                }
                Task::none()
            }

            Message::TunnelRemove(tunnel_name) => {
                self.logger.log_print(&format!("Remove tunnel '{}'", tunnel_name));
                match self.tunnel_manager.remove_tunnel(&tunnel_name) {
                    Ok(_) => {
                        // Save the updated tunnels list
                        if let Err(e) =
                            TunnelManager::save_tunnels(&self.tunnels_file, self.tunnel_manager.get_tunnels())
                        {
                            self.logger.log_print(&format!("Error saving tunnels: {}", e));
                        } else {
                            notifications::notify_tunnel_removed(&tunnel_name);
                        }
                    }
                    Err(e) => {
                        self.logger.log_print(&format!(
                            "Error removing tunnel '{}': {}",
                            tunnel_name, e
                        ));
                    }
                }
                self.update(Message::UpdateTrayMenu)
            }

            Message::Quit => {
                self.logger.log_print("Quit selected from tray menu");
                let manager = &mut self.tunnel_manager;
                manager.cleanup();
                let _ = manager;
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

            // Unified tunnel form field update handler
            Message::TunnelFormFieldChanged(window_id, field) => {
                self.update_tunnel_form_field(window_id, field);
                Task::none()
            }

            Message::TunnelFormBrowsePrivateKey(window_id) => {
                if let Some(path) = windows::create_tunnel::browse_for_private_key() {
                    self.update_tunnel_form_field(
                        window_id,
                        TunnelFormField::PrivateKey(path),
                    );
                }
                Task::none()
            }

            Message::TunnelFormTest(window_id) => {
                // Get the window data and test the connection
                let window_type = self.windows.get_mut(&window_id);
                if window_type.is_none() {
                    return Task::none();
                }

                let extra_height = match window_type.unwrap() {
                    WindowType::TunnelForm {
                        name, local_host, local_port, remote_host, remote_port,
                        ssh_user, ssh_host, ssh_port, private_key,
                        error_message, test_message, ..
                    } => {
                        *error_message = None;
                        *test_message = None;
                        match windows::create_tunnel::validate_and_create_tunnel(
                            name, local_host, local_port, remote_host, remote_port,
                            ssh_user, ssh_host, ssh_port, private_key,
                        ) {
                            Ok(tunnel) => {
                                match TunnelManager::test_tunnel(&tunnel) {
                                    Ok(success_msg) => *test_message = Some(success_msg),
                                    Err(err) => *test_message = Some(format!("{}", err)),
                                }
                                test_message.as_ref().map(|msg| (msg.len() / 60).max(1) as f32 * 20.0 + 40.0).unwrap_or(0.0)
                            }
                            Err(err) => {
                                *error_message = Some(format!("{}", err));
                                error_message.as_ref().map(|msg| (msg.len() / 60).max(1) as f32 * 20.0 + 40.0).unwrap_or(0.0)
                            }
                        }
                    }
                    _ => return Task::none(),
                };

                window::resize(window_id, Size::new(500.0, 640.0 + extra_height))
            }

            Message::TunnelFormSubmit(window_id) => {
                self.handle_tunnel_form_submit(window_id)
            }

            Message::TunnelFormCancel(window_id) => window::close(window_id),

            Message::UpdateTrayMenu => {
                if let (Some(tray_icon), Some(_)) = (&mut self.tray_icon, &self.menu_ids) {
                    let manager = &self.tunnel_manager;
                    let tunnels = manager.get_tunnels().clone();
                    let _ = manager;
                    let tunnel_statuses: Vec<(String, crate::tunnels::TunnelStatus)> = manager
                        .get_tunnels()
                        .iter()
                        .map(|t| (t.name.clone(), manager.get_tunnel_status(&t.name)))
                        .collect();
                    match systemtray::update_tray_menu(tray_icon, &tunnels, &tunnel_statuses) {
                        Ok(new_ids) => {
                            self.menu_ids = Some(new_ids);
                        }
                        Err(e) => {
                            self.logger.log_print(&format!("Error updating tray menu: {}", e));
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
                    windows::about::view().map(|msg| match msg {})
                }
                WindowType::TunnelForm {
                    mode,
                    name, local_host, local_port, remote_host, remote_port,
                    ssh_user, ssh_host, ssh_port, private_key,
                    error_message, test_message,
                } => {
                    windows::create_tunnel::view(
                        mode,
                        name, local_host, local_port,
                        remote_host, remote_port,
                        ssh_user, ssh_host, ssh_port,
                        private_key,
                        error_message,
                        test_message,
                    )
                    .map(move |msg| self.map_tunnel_form_message(window_id, msg))
                }
            }
        } else {
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

        // Tunnel status monitoring subscription
        struct TunnelStatusMonitor;
        let mut status_receiver = self.status_receiver.resubscribe();
        let status_subscription = Subscription::run_with_id(
            std::any::TypeId::of::<TunnelStatusMonitor>(),
            iced::stream::channel(100, move |mut output| async move {
                loop {
                    let update_opt = status_receiver.try_recv().ok();
                    if let Some(update) = update_opt {
                        let _ = output.send(update).await;
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            })
        ).map(Message::TunnelStatusUpdate);

        Subscription::batch(vec![window_events, tray_subscription, status_subscription])
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

    /// Handles tray menu events and dispatches appropriate messages
    fn handle_tray_menu_event(&mut self, event: MenuEvent) -> Task<Message> {
        // log_print(&format!("Received tray menu event: {:?}", event.id));
        
        let Some(menu_ids) = &self.menu_ids else {
            return Task::none();
        };

        // Check static menu items
        if event.id == menu_ids.create {
            return self.update(Message::OpenCreateTunnel);
        }
        if event.id == menu_ids.about {
            return self.update(Message::OpenAbout);
        }
        if event.id == menu_ids.quit {
            return self.update(Message::Quit);
        }

        // Check tunnel-specific menu items
        for (tunnel_name, menu_id) in &menu_ids.tunnel_connect {
            if event.id == *menu_id {
                return self.update(Message::TunnelConnect(tunnel_name.clone()));
            }
        }
        for (tunnel_name, menu_id) in &menu_ids.tunnel_disconnect {
            if event.id == *menu_id {
                return self.update(Message::TunnelDisconnect(tunnel_name.clone()));
            }
        }
        for (tunnel_name, menu_id) in &menu_ids.tunnel_open_web {
            if event.id == *menu_id {
                return self.update(Message::TunnelOpenWeb(tunnel_name.clone()));
            }
        }
        for (tunnel_name, menu_id) in &menu_ids.tunnel_edit {
            if event.id == *menu_id {
                return self.update(Message::TunnelEdit(tunnel_name.clone()));
            }
        }
        for (tunnel_name, menu_id) in &menu_ids.tunnel_remove {
            if event.id == *menu_id {
                return self.update(Message::TunnelRemove(tunnel_name.clone()));
            }
        }

        Task::none()
    }

    /// Maps tunnel form messages from the view to app messages with window ID
    fn map_tunnel_form_message(&self, window_id: window::Id, msg: windows::create_tunnel::Message) -> Message {
        match msg {
            windows::create_tunnel::Message::NameChanged(v) => 
                Message::TunnelFormFieldChanged(window_id, TunnelFormField::Name(v)),
            windows::create_tunnel::Message::LocalHostChanged(v) => 
                Message::TunnelFormFieldChanged(window_id, TunnelFormField::LocalHost(v)),
            windows::create_tunnel::Message::LocalPortChanged(v) => 
                Message::TunnelFormFieldChanged(window_id, TunnelFormField::LocalPort(v)),
            windows::create_tunnel::Message::RemoteHostChanged(v) => 
                Message::TunnelFormFieldChanged(window_id, TunnelFormField::RemoteHost(v)),
            windows::create_tunnel::Message::RemotePortChanged(v) => 
                Message::TunnelFormFieldChanged(window_id, TunnelFormField::RemotePort(v)),
            windows::create_tunnel::Message::SshUserChanged(v) => 
                Message::TunnelFormFieldChanged(window_id, TunnelFormField::SshUser(v)),
            windows::create_tunnel::Message::SshHostChanged(v) => 
                Message::TunnelFormFieldChanged(window_id, TunnelFormField::SshHost(v)),
            windows::create_tunnel::Message::SshPortChanged(v) => 
                Message::TunnelFormFieldChanged(window_id, TunnelFormField::SshPort(v)),
            windows::create_tunnel::Message::PrivateKeyChanged(v) => 
                Message::TunnelFormFieldChanged(window_id, TunnelFormField::PrivateKey(v)),
            windows::create_tunnel::Message::BrowsePrivateKey => 
                Message::TunnelFormBrowsePrivateKey(window_id),
            windows::create_tunnel::Message::Test => 
                Message::TunnelFormTest(window_id),
            windows::create_tunnel::Message::Create => 
                Message::TunnelFormSubmit(window_id),
            windows::create_tunnel::Message::Cancel => 
                Message::TunnelFormCancel(window_id),
        }
    }

    /// Updates a form field in the tunnel form window
    fn update_tunnel_form_field(&mut self, window_id: window::Id, field: TunnelFormField) {
        if let Some(window_type) = self.windows.get_mut(&window_id) {
            match window_type {
                WindowType::TunnelForm {
                    name, local_host, local_port, remote_host, remote_port,
                    ssh_user, ssh_host, ssh_port, private_key, ..
                } => {
                    match field {
                        TunnelFormField::Name(v) => *name = v,
                        TunnelFormField::LocalHost(v) => *local_host = v,
                        TunnelFormField::LocalPort(v) => *local_port = v,
                        TunnelFormField::RemoteHost(v) => *remote_host = v,
                        TunnelFormField::RemotePort(v) => *remote_port = v,
                        TunnelFormField::SshUser(v) => *ssh_user = v,
                        TunnelFormField::SshHost(v) => *ssh_host = v,
                        TunnelFormField::SshPort(v) => *ssh_port = v,
                        TunnelFormField::PrivateKey(v) => *private_key = v,
                    }
                }
                _ => {}
            }
        }
    }

    /// Handles tunnel form submission for both create and edit modes
    fn handle_tunnel_form_submit(&mut self, window_id: window::Id) -> Task<Message> {
        let window_type = self.windows.get_mut(&window_id);
        if window_type.is_none() {
            return Task::none();
        }
        match window_type.unwrap() {
            WindowType::TunnelForm {
                mode,
                name, local_host, local_port, remote_host, remote_port,
                ssh_user, ssh_host, ssh_port, private_key,
                error_message, ..
            } => {
                match windows::create_tunnel::validate_and_create_tunnel(
                    name, local_host, local_port, remote_host, remote_port,
                    ssh_user, ssh_host, ssh_port, private_key,
                ) {
                    Ok(mut tunnel) => {
                        let manager = &mut self.tunnel_manager;
                        match mode {
                            windows::FormMode::Create => {
                                // log_print(&format!("Saving new tunnel: {}", tunnel.name));
                                manager.add_tunnel(tunnel.clone());
                                if let Err(_e) = TunnelManager::save_tunnels(&self.tunnels_file, manager.get_tunnels()) {
                                    // log_print(&format!("Error saving tunnels: {}", _e));
                                } else {
                                    notifications::notify_tunnel_created(&tunnel.name);
                                }
                            }
                            windows::FormMode::Edit { tunnel_id } => {
                                // log_print(&format!("Updating tunnel: {}", tunnel.name));
                                tunnel.id = tunnel_id.clone();
                                if let Err(e) = manager.update_tunnel(tunnel_id, tunnel.clone()) {
                                    // log_print(&format!("Error updating tunnel: {}", e));
                                    *error_message = Some(format!("Error updating tunnel: {}", e));
                                    let _ = manager;
                                    return Task::none();
                                }
                                if let Err(_e) = TunnelManager::save_tunnels(&self.tunnels_file, manager.get_tunnels()) {
                                    // log_print(&format!("Error saving tunnels: {}", _e));
                                }
                            }
                        }
                        let _ = manager;
                        Task::batch(vec![
                            self.update(Message::UpdateTrayMenu),
                            window::close(window_id),
                        ])
                    }
                    Err(err) => {
                        *error_message = Some(format!("{}", err));
                        let extra_height = error_message.as_ref()
                            .map(|msg| (msg.len() / 60).max(1) as f32 * 20.0 + 40.0)
                            .unwrap_or(0.0);
                        window::resize(window_id, Size::new(500.0, 640.0 + extra_height))
                    }
                }
            }
            _ => Task::none(),
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
