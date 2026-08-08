use std::collections::BTreeMap;
use std::path::PathBuf;
use tokio::sync::broadcast;
use tracing::{info, warn, error};
use tray_icon::menu::MenuEvent;
#[cfg(not(target_os = "linux"))]
use tray_icon::TrayIcon;

use iced::futures::SinkExt;
use iced::window;
use iced::{Element, Size, Subscription, Task};

use crate::config;
use crate::logs;
use crate::notifications;
use crate::platform;
use crate::systemtray::{self, TrayMenuIds};
use crate::tunnels::{StatusUpdate, TunnelManager, TunnelStatus};
use crate::windows::create_tunnel::TunnelFormField;
use crate::windows::{self, WindowType};

pub struct App {
    windows: BTreeMap<window::Id, WindowType>,
    tunnel_manager: TunnelManager,
    tunnels_file: PathBuf,
    #[cfg(not(target_os = "linux"))]
    tray_icon: Option<TrayIcon>,
    #[cfg(target_os = "linux")]
    linux_tray: Option<platform::linux::LinuxTrayHandle>,
    menu_ids: Option<TrayMenuIds>,
    _logging_guard: tracing_appender::non_blocking::WorkerGuard,
    status_receiver: broadcast::Receiver<StatusUpdate>,
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

    // Form messages
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
        let (config_path, logs_dir) = match config::init_config() {
            Ok(paths) => paths,
            Err(e) => {
                eprintln!("Error initializing configuration: {}", e);
                std::process::exit(1);
            }
        };

        let logging_guard = logs::init_logging(&logs_dir);

        info!("Drill - Multi-Platform Tunnel Manager");
        info!("Platform: {}", platform::get_platform_name());
        info!("Configuration loaded from: {}", config_path.display());

        let tunnels_file = match config::get_tunnels_file_path() {
            Ok(path) => path,
            Err(e) => {
                error!("Error getting tunnels file path: {}", e);
                std::process::exit(1);
            }
        };

        let tunnels = match TunnelManager::load_tunnels(&tunnels_file) {
            Ok(t) => t,
            Err(e) => {
                warn!("Error loading tunnels (using empty list): {}", e);
                Vec::new()
            }
        };

        let mut tunnel_manager = TunnelManager::new();
        tunnel_manager.set_tunnels(tunnels.clone());

        let (status_tx, status_rx) = broadcast::channel(100);
        tunnel_manager.set_status_channel(status_tx);
        let status_receiver = status_rx;

        let tunnel_statuses: Vec<(String, TunnelStatus)> = tunnel_manager
            .get_tunnels()
            .iter()
            .map(|t| (t.name.clone(), tunnel_manager.get_tunnel_status(&t.name)))
            .collect();

        #[cfg(not(target_os = "linux"))]
        let (tray_icon, menu_ids) = match systemtray::init_tray(&tunnels, &tunnel_statuses) {
            Ok((icon, ids)) => (Some(icon), Some(ids)),
            Err(e) => {
                error!("Error initializing system tray: {}", e);
                std::process::exit(1);
            }
        };

        #[cfg(target_os = "linux")]
        let (linux_tray, menu_ids) = match platform::linux::spawn_linux_tray(tunnels.clone(), tunnel_statuses.clone()) {
            Ok((handle, ids)) => (Some(handle), Some(ids)),
            Err(e) => {
                error!("Error initializing Linux system tray: {}", e);
                (None, None)
            }
        };

        info!("Drill initialization complete. Running...");

        (
            Self {
                windows: BTreeMap::new(),
                tunnel_manager,
                tunnels_file,
                #[cfg(not(target_os = "linux"))]
                tray_icon,
                #[cfg(target_os = "linux")]
                linux_tray,
                menu_ids,
                _logging_guard: logging_guard,
                status_receiver,
            },
            Task::none(),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::TrayMenuEvent(event) => self.handle_tray_menu_event(event),

            Message::OpenAbout => {
                if let Some((window_id, _)) = self
                    .windows
                    .iter()
                    .find(|(_, wt)| matches!(wt, WindowType::About))
                {
                    return window::gain_focus(*window_id);
                }
                let (id, open) = window::open(window::Settings {
                    size: Size::new(400.0, 300.0),
                    resizable: false,
                    ..window::Settings::default()
                });
                open.then(move |_| Task::done(Message::WindowOpened(id, WindowType::About)))
            }

            Message::OpenCreateTunnel => {
                if let Some((window_id, _wt)) = self.windows.iter().find(|(_, wt)| {
                    matches!(
                        wt,
                        WindowType::TunnelForm(windows::create_tunnel::TunnelFormState {
                            mode: windows::FormMode::Create,
                            ..
                        })
                    )
                }) {
                    return window::gain_focus(*window_id);
                }
                let (id, open) = window::open(window::Settings {
                    size: Size::new(500.0, 700.0),
                    resizable: true,
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
                self.tunnel_manager.update_status_from_event(&update);
                match update {
                    StatusUpdate::Connecting(ref tunnel_name) => {
                        info!("Tunnel '{}' is connecting...", tunnel_name);
                    }
                    StatusUpdate::Connected(ref tunnel_name) => {
                        info!("Tunnel '{}' connected successfully", tunnel_name);
                        let _ = notifications::notify_tunnel_connected(tunnel_name);
                        return self.update(Message::UpdateTrayMenu);
                    }
                    StatusUpdate::Error(ref tunnel_name, ref err) => {
                        error!("Tunnel '{}' error: {}", tunnel_name, err);
                        notifications::notify_tunnel_error(tunnel_name, err);
                        return self.update(Message::UpdateTrayMenu);
                    }
                    StatusUpdate::Disconnected(ref tunnel_name) => {
                        info!("Tunnel '{}' disconnected", tunnel_name);
                        return self.update(Message::UpdateTrayMenu);
                    }
                }
                Task::none()
            }

            Message::TunnelConnect(tunnel_name) => {
                if let Some(tunnel) = self
                    .tunnel_manager
                    .get_tunnels()
                    .iter()
                    .find(|t| t.name == tunnel_name)
                    .cloned()
                {
                    if let Err(e) = self.tunnel_manager.start_tunnel(&tunnel) {
                        error!("Error starting tunnel '{}': {}", tunnel_name, e);
                        notifications::notify_tunnel_error(&tunnel_name, &e.to_string());
                    }
                }
                self.update(Message::UpdateTrayMenu)
            }

            Message::TunnelDisconnect(tunnel_name) => {
                if let Err(e) = self.tunnel_manager.stop_tunnel(&tunnel_name) {
                    error!("Error stopping tunnel '{}': {}", tunnel_name, e);
                } else {
                    notifications::notify_tunnel_disconnected(&tunnel_name);
                }
                self.update(Message::UpdateTrayMenu)
            }

            Message::TunnelOpenWeb(tunnel_name) => {
                if let Some(tunnel) = self
                    .tunnel_manager
                    .get_tunnels()
                    .iter()
                    .find(|t| t.name == tunnel_name)
                {
                    if let Some(ref web_url) = tunnel.web_url {
                        if !web_url.trim().is_empty() {
                            info!("Opening Web URL: {}", web_url);
                            if let Err(e) = open_browser_url(web_url) {
                                error!("Error opening URL '{}': {}", web_url, e);
                            }
                        } else {
                            warn!("No Web URL defined for tunnel '{}'", tunnel_name);
                        }
                    } else {
                        warn!("No Web URL defined for tunnel '{}'", tunnel_name);
                    }
                }
                Task::none()
            }

            Message::TunnelEdit(tunnel_name) => {
                if let Some((window_id, _wt)) = self.windows.iter().find(|(_, wt)| {
                    matches!(wt, WindowType::TunnelForm(windows::create_tunnel::TunnelFormState { mode: windows::FormMode::Edit { tunnel_id }, .. }) if {
                        self.tunnel_manager.get_tunnels().iter().any(|t| t.name == tunnel_name && &t.id == tunnel_id)
                    })
                }) {
                    return window::gain_focus(*window_id);
                }
                if let Some(tunnel) = self
                    .tunnel_manager
                    .get_tunnels()
                    .iter()
                    .find(|t| t.name == tunnel_name)
                {
                    let tunnel_clone = tunnel.clone();
                    let (id, open) = window::open(window::Settings {
                        size: Size::new(500.0, 700.0),
                        resizable: true,
                        ..window::Settings::default()
                    });
                    return open.then(move |_| {
                        Task::done(Message::WindowOpened(
                            id,
                            WindowType::new_tunnel_form_edit(&tunnel_clone),
                        ))
                    });
                }
                Task::none()
            }

            Message::TunnelRemove(tunnel_name) => {
                info!("Removing tunnel '{}'", tunnel_name);
                match self.tunnel_manager.remove_tunnel(&tunnel_name) {
                    Ok(_) => {
                        if let Err(e) = TunnelManager::save_tunnels(
                            &self.tunnels_file,
                            self.tunnel_manager.get_tunnels(),
                        ) {
                            error!("Error saving tunnels: {}", e);
                        } else {
                            notifications::notify_tunnel_removed(&tunnel_name);
                        }
                    }
                    Err(e) => {
                        error!("Error removing tunnel '{}': {}", tunnel_name, e);
                    }
                }
                self.update(Message::UpdateTrayMenu)
            }

            Message::Quit => {
                info!("Quit requested from tray menu");
                self.tunnel_manager.cleanup();
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

            Message::TunnelFormFieldChanged(window_id, field) => {
                if let Some(WindowType::TunnelForm(state)) = self.windows.get_mut(&window_id) {
                    state.update_field(field);
                }
                Task::none()
            }

            Message::TunnelFormBrowsePrivateKey(window_id) => {
                if let Some(path) = windows::create_tunnel::browse_for_private_key() {
                    if let Some(WindowType::TunnelForm(state)) = self.windows.get_mut(&window_id) {
                        state.update_field(TunnelFormField::PrivateKey(path));
                    }
                }
                Task::none()
            }

            Message::TunnelFormTest(window_id) => {
                if let Some(WindowType::TunnelForm(state)) = self.windows.get_mut(&window_id) {
                    state.error_message = None;
                    state.test_message = None;
                    match windows::create_tunnel::validate_and_create_tunnel(state) {
                        Ok(tunnel) => match TunnelManager::test_tunnel(&tunnel) {
                            Ok(success_msg) => state.test_message = Some(success_msg),
                            Err(err) => state.test_message = Some(format!("{}", err)),
                        },
                        Err(err) => state.error_message = Some(err),
                    }
                }
                Task::none()
            }

            Message::TunnelFormSubmit(window_id) => self.handle_tunnel_form_submit(window_id),

            Message::TunnelFormCancel(window_id) => window::close(window_id),

            Message::UpdateTrayMenu => {
                let tunnels = self.tunnel_manager.get_tunnels().clone();
                let tunnel_statuses: Vec<(String, TunnelStatus)> = self
                    .tunnel_manager
                    .get_tunnels()
                    .iter()
                    .map(|t| (t.name.clone(), self.tunnel_manager.get_tunnel_status(&t.name)))
                    .collect();

                #[cfg(not(target_os = "linux"))]
                if let (Some(tray_icon), Some(_)) = (&mut self.tray_icon, &self.menu_ids) {
                    match systemtray::update_tray_menu(tray_icon, &tunnels, &tunnel_statuses) {
                        Ok(new_ids) => {
                            self.menu_ids = Some(new_ids);
                        }
                        Err(e) => {
                            error!("Error updating tray menu: {}", e);
                        }
                    }
                }

                #[cfg(target_os = "linux")]
                if let (Some(linux_tray), Some(_)) = (&mut self.linux_tray, &self.menu_ids) {
                    match linux_tray.update_menu(&tunnels, &tunnel_statuses) {
                        Ok(new_ids) => {
                            self.menu_ids = Some(new_ids);
                        }
                        Err(e) => {
                            error!("Error updating Linux tray menu: {}", e);
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
                WindowType::About => windows::about::view().map(|msg| match msg {}),
                WindowType::TunnelForm(state) => windows::create_tunnel::view(state)
                    .map(move |msg| self.map_tunnel_form_message(window_id, msg)),
            }
        } else {
            iced::widget::text("Window not found").into()
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let window_events = iced::event::listen_with(|event, _status, id| match event {
            iced::Event::Window(window_event) => match window_event {
                iced::window::Event::Closed => Some(Message::WindowClosed(id)),
                _ => None,
            },
            _ => None,
        });

        // Non-blocking tray menu events subscription
        struct TrayEventsPoll;
        let tray_subscription = Subscription::run_with_id(
            std::any::TypeId::of::<TrayEventsPoll>(),
            iced::stream::channel(100, |mut output| async move {
                let menu_channel = MenuEvent::receiver();
                loop {
                    while let Ok(event) = menu_channel.try_recv() {
                        let _ = output.send(event).await;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(16)).await;
                }
            }),
        )
        .map(Message::TrayMenuEvent);

        // Event-driven tunnel status monitor subscription (Tokio broadcast recv, no polling sleep!)
        struct TunnelStatusMonitor;
        let mut status_receiver = self.status_receiver.resubscribe();
        let status_subscription = Subscription::run_with_id(
            std::any::TypeId::of::<TunnelStatusMonitor>(),
            iced::stream::channel(100, move |mut output| async move {
                while let Ok(update) = status_receiver.recv().await {
                    let _ = output.send(update).await;
                }
            }),
        )
        .map(Message::TunnelStatusUpdate);

        Subscription::batch(vec![window_events, tray_subscription, status_subscription])
    }

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

    fn handle_tray_menu_event(&mut self, event: MenuEvent) -> Task<Message> {
        let Some(menu_ids) = &self.menu_ids else {
            return Task::none();
        };

        if event.id == menu_ids.create {
            return self.update(Message::OpenCreateTunnel);
        }
        if event.id == menu_ids.about {
            return self.update(Message::OpenAbout);
        }
        if event.id == menu_ids.quit {
            return self.update(Message::Quit);
        }

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

    fn map_tunnel_form_message(
        &self,
        window_id: window::Id,
        msg: windows::create_tunnel::Message,
    ) -> Message {
        match msg {
            windows::create_tunnel::Message::FieldChanged(field) => {
                Message::TunnelFormFieldChanged(window_id, field)
            }
            windows::create_tunnel::Message::BrowsePrivateKey => {
                Message::TunnelFormBrowsePrivateKey(window_id)
            }
            windows::create_tunnel::Message::Test => Message::TunnelFormTest(window_id),
            windows::create_tunnel::Message::Create => Message::TunnelFormSubmit(window_id),
            windows::create_tunnel::Message::Cancel => Message::TunnelFormCancel(window_id),
        }
    }

    fn handle_tunnel_form_submit(&mut self, window_id: window::Id) -> Task<Message> {
        let Some(WindowType::TunnelForm(state)) = self.windows.get_mut(&window_id) else {
            return Task::none();
        };

        match windows::create_tunnel::validate_and_create_tunnel(state) {
            Ok(mut tunnel) => {
                let mode = state.mode.clone();
                match mode {
                    windows::FormMode::Create => {
                        self.tunnel_manager.add_tunnel(tunnel.clone());
                        if let Err(e) = TunnelManager::save_tunnels(
                            &self.tunnels_file,
                            self.tunnel_manager.get_tunnels(),
                        ) {
                            error!("Error saving tunnels: {}", e);
                        } else {
                            notifications::notify_tunnel_created(&tunnel.name);
                        }
                    }
                    windows::FormMode::Edit { tunnel_id } => {
                        tunnel.id = tunnel_id.clone();
                        if let Err(e) = self.tunnel_manager.update_tunnel(&tunnel_id, tunnel.clone()) {
                            error!("Error updating tunnel: {}", e);
                            if let Some(WindowType::TunnelForm(state)) = self.windows.get_mut(&window_id) {
                                state.error_message = Some(format!("Error updating tunnel: {}", e));
                            }
                            return Task::none();
                        }
                        if let Err(e) = TunnelManager::save_tunnels(
                            &self.tunnels_file,
                            self.tunnel_manager.get_tunnels(),
                        ) {
                            error!("Error saving tunnels: {}", e);
                        }
                    }
                }
                Task::batch(vec![
                    self.update(Message::UpdateTrayMenu),
                    window::close(window_id),
                ])
            }
            Err(err) => {
                if let Some(WindowType::TunnelForm(state)) = self.windows.get_mut(&window_id) {
                    state.error_message = Some(err);
                }
                Task::none()
            }
        }
    }
}

pub fn open_browser_url(raw_url: &str) -> Result<(), String> {
    let mut url = raw_url.trim().to_string();
    if !url.starts_with("http://") && !url.starts_with("https://") {
        url = format!("http://{}", url);
    }

    if open::that(&url).is_ok() {
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        let commands = [
            ("xdg-open", vec![url.as_str()]),
            ("gio", vec!["open", url.as_str()]),
            ("x-www-browser", vec![url.as_str()]),
            ("sensible-browser", vec![url.as_str()]),
            ("firefox", vec![url.as_str()]),
            ("google-chrome", vec![url.as_str()]),
            ("chromium", vec![url.as_str()]),
            ("brave-browser", vec![url.as_str()]),
        ];

        for (cmd, args) in commands {
            if let Ok(child) = std::process::Command::new(cmd)
                .args(&args)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                std::mem::forget(child);
                return Ok(());
            }
        }
    }

    Err(format!("Could not launch browser for URL: {}", url))
}
