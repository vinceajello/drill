use iced::{
    widget::{button, column, container, text, text_input},
    window, Element, Length, Size, Task,
};

fn main() -> iced::Result {
    iced::application(
        "Create New Tunnel",
        CreateTunnelDialog::update,
        CreateTunnelDialog::view,
    )
    .window_size(Size::new(500.0, 640.0))
    .resizable(false)
    .run()
}

struct CreateTunnelDialog {
    name: String,
    local_host: String,
    local_port: String,
    remote_host: String,
    remote_port: String,
    ssh_user: String,
    ssh_host: String,
    ssh_port: String,
    error_message: Option<String>,
}

#[derive(Debug, Clone)]
enum Message {
    NameChanged(String),
    LocalHostChanged(String),
    LocalPortChanged(String),
    RemoteHostChanged(String),
    RemotePortChanged(String),
    SshUserChanged(String),
    SshHostChanged(String),
    SshPortChanged(String),
    Create,
    Cancel,
}

impl Default for CreateTunnelDialog {
    fn default() -> Self {
        CreateTunnelDialog {
            name: String::new(),
            local_host: "localhost".to_string(),
            local_port: String::new(),
            remote_host: String::new(),
            remote_port: String::new(),
            ssh_user: String::new(),
            ssh_host: String::new(),
            ssh_port: "22".to_string(),
            error_message: None,
        }
    }
}

impl CreateTunnelDialog {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::NameChanged(value) => {
                self.name = value;
                Task::none()
            }
            Message::LocalHostChanged(value) => {
                self.local_host = value;
                Task::none()
            }
            Message::LocalPortChanged(value) => {
                self.local_port = value;
                Task::none()
            }
            Message::RemoteHostChanged(value) => {
                self.remote_host = value;
                Task::none()
            }
            Message::RemotePortChanged(value) => {
                self.remote_port = value;
                Task::none()
            }
            Message::SshUserChanged(value) => {
                self.ssh_user = value;
                Task::none()
            }
            Message::SshHostChanged(value) => {
                self.ssh_host = value;
                Task::none()
            }
            Message::SshPortChanged(value) => {
                self.ssh_port = value;
                Task::none()
            }
            Message::Create => {
                // Validate inputs
                if self.name.trim().is_empty() {
                    self.error_message = Some("Name is required".to_string());
                    return Task::none();
                }

                // TODO: Save tunnel data to file or stdout for parent process
                println!("TUNNEL_CREATED:{}", self.name);

                window::get_latest().and_then(window::close)
            }
            Message::Cancel => window::get_latest().and_then(window::close),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let mut content = column![
            text("Create New Tunnel").size(20),
            text("").size(8),
            text("Tunnel Name:").size(14),
            text_input("Enter tunnel name", &self.name)
                .on_input(Message::NameChanged)
                .padding(8),
            text("").size(4),
            text("Local:").size(14),
            text_input("localhost", &self.local_host)
                .on_input(Message::LocalHostChanged)
                .padding(8),
            text_input("Local port (e.g., 8080)", &self.local_port)
                .on_input(Message::LocalPortChanged)
                .padding(8),
            text("").size(4),
            text("Remote:").size(14),
            text_input("Remote host", &self.remote_host)
                .on_input(Message::RemoteHostChanged)
                .padding(8),
            text_input("Remote port", &self.remote_port)
                .on_input(Message::RemotePortChanged)
                .padding(8),
            text("").size(4),
            text("SSH Connection:").size(14),
            text_input("SSH user", &self.ssh_user)
                .on_input(Message::SshUserChanged)
                .padding(8),
            text_input("SSH host", &self.ssh_host)
                .on_input(Message::SshHostChanged)
                .padding(8),
            text_input("SSH port", &self.ssh_port)
                .on_input(Message::SshPortChanged)
                .padding(8),
        ]
        .spacing(5)
        .padding(20);

        if let Some(error) = &self.error_message {
            content = content.push(text("").size(4));
            content = content.push(text(error).color(iced::Color::from_rgb(0.8, 0.0, 0.0)));
        }

        content = content.push(text("").size(8));
        content = content.push(
            iced::widget::row![
                button("Cancel").on_press(Message::Cancel).padding(8),
                text(" "),
                button("Create").on_press(Message::Create).padding(8),
            ]
            .spacing(10),
        );

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
