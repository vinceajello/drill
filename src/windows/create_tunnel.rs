use crate::tunnels::Tunnel;
use super::FormMode;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Column};
use iced::{Element, Length};

#[derive(Debug, Clone, Default)]
pub struct TunnelFormState {
    pub mode: FormMode,
    pub name: String,
    pub local_host: String,
    pub local_port: String,
    pub remote_host: String,
    pub remote_port: String,
    pub ssh_user: String,
    pub ssh_host: String,
    pub ssh_port: String,
    pub private_key: String,
    pub web_url: String,
    pub error_message: Option<String>,
    pub test_message: Option<String>,
}

impl TunnelFormState {
    pub fn update_field(&mut self, field: TunnelFormField) {
        match field {
            TunnelFormField::Name(v) => self.name = v,
            TunnelFormField::LocalHost(v) => self.local_host = v,
            TunnelFormField::LocalPort(v) => self.local_port = v,
            TunnelFormField::RemoteHost(v) => self.remote_host = v,
            TunnelFormField::RemotePort(v) => self.remote_port = v,
            TunnelFormField::SshUser(v) => self.ssh_user = v,
            TunnelFormField::SshHost(v) => self.ssh_host = v,
            TunnelFormField::SshPort(v) => self.ssh_port = v,
            TunnelFormField::PrivateKey(v) => self.private_key = v,
            TunnelFormField::WebUrl(v) => self.web_url = v,
        }
    }
}

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
    WebUrl(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    FieldChanged(TunnelFormField),
    BrowsePrivateKey,
    Test,
    Create,
    Cancel,
}

pub fn view<'a>(state: &'a TunnelFormState) -> Element<'a, Message> {
    let title = match &state.mode {
        FormMode::Edit { .. } => "Edit Tunnel",
        FormMode::Create => "Drill New Tunnel",
    };
    let mut content: Column<'a, Message> = column![
        text(title).size(20),
        text("").size(8),
        text("Tunnel Name:").size(14),
        text_input("Enter tunnel name", &state.name)
            .on_input(|v| Message::FieldChanged(TunnelFormField::Name(v)))
            .padding(8),
        text("").size(4),
        row![
            column![
                text("Local Host").size(12),
                text_input("localhost", &state.local_host)
                    .on_input(|v| Message::FieldChanged(TunnelFormField::LocalHost(v)))
                    .padding(8),
            ]
            .spacing(2)
            .width(Length::Fill),
            text(" ").width(Length::Fixed(10.0)),
            column![
                text("Local Port").size(12),
                text_input("Port (e.g., 8080)", &state.local_port)
                    .on_input(|v| Message::FieldChanged(TunnelFormField::LocalPort(v)))
                    .padding(8),
            ]
            .spacing(2)
            .width(Length::Fill),
        ],
        text("").size(4),
        row![
            column![
                text("Remote Host").size(12),
                text_input("Remote host", &state.remote_host)
                    .on_input(|v| Message::FieldChanged(TunnelFormField::RemoteHost(v)))
                    .padding(8),
            ]
            .spacing(2)
            .width(Length::Fill),
            text(" ").width(Length::Fixed(10.0)),
            column![
                text("Remote Port").size(12),
                text_input("Remote port", &state.remote_port)
                    .on_input(|v| Message::FieldChanged(TunnelFormField::RemotePort(v)))
                    .padding(8),
            ]
            .spacing(2)
            .width(Length::Fill),
        ],
        text("").size(4),
        text("SSH Connection:").size(14),
        text_input("SSH user", &state.ssh_user)
            .on_input(|v| Message::FieldChanged(TunnelFormField::SshUser(v)))
            .padding(8),
        row![
            column![
                text("SSH Host").size(12),
                text_input("SSH host", &state.ssh_host)
                    .on_input(|v| Message::FieldChanged(TunnelFormField::SshHost(v)))
                    .padding(8),
            ]
            .spacing(2)
            .width(Length::Fill),
            text(" ").width(Length::Fixed(10.0)),
            column![
                text("SSH Port").size(12),
                text_input("Port (e.g., 22)", &state.ssh_port)
                    .on_input(|v| Message::FieldChanged(TunnelFormField::SshPort(v)))
                    .padding(8),
            ]
            .spacing(2)
            .width(Length::Fill),
        ],
        text("").size(4),
        text("Private Key (optional)").size(12),
        row![
            text_input("Path to private key file", &state.private_key)
                .on_input(|v| Message::FieldChanged(TunnelFormField::PrivateKey(v)))
                .padding(8)
                .width(Length::Fill),
            text(" ").width(Length::Fixed(8.0)),
            button("Browse")
                .on_press(Message::BrowsePrivateKey)
                .padding(8),
        ]
        .align_y(iced::Alignment::Center),
        text("").size(4),
        text("Web URL (optional)").size(12),
        text_input("e.g. http://localhost:8080", &state.web_url)
            .on_input(|v| Message::FieldChanged(TunnelFormField::WebUrl(v)))
            .padding(8),
    ]
    .spacing(5)
    .padding(20);

    if let Some(error) = &state.error_message {
        content = content.push(text("").size(4));
        content = content.push(
            text(error)
                .color(iced::Color::from_rgb(0.8, 0.0, 0.0))
                .wrapping(iced::widget::text::Wrapping::Word),
        );
    }

    if let Some(test_msg) = &state.test_message {
        content = content.push(text("").size(4));
        if test_msg.starts_with("Success") || test_msg.starts_with("✓") {
            content = content.push(
                text(test_msg)
                    .color(iced::Color::from_rgb(0.0, 0.6, 0.0))
                    .wrapping(iced::widget::text::Wrapping::Word),
            );
        } else {
            content = content.push(
                text(test_msg)
                    .color(iced::Color::from_rgb(0.8, 0.5, 0.0))
                    .wrapping(iced::widget::text::Wrapping::Word),
            );
        }
    }

    content = content.push(text("").size(8));
    let is_edit_mode = matches!(&state.mode, FormMode::Edit { .. });
    let action_button_text = if is_edit_mode { "Save" } else { "Create" };
    content = content.push(
        row![
            button("Cancel").on_press(Message::Cancel).padding(8),
            text(" "),
            button("Test").on_press(Message::Test).padding(8),
            text(" "),
            button(action_button_text).on_press(Message::Create).padding(8),
        ]
        .spacing(10),
    );

    // Scrollable container for dynamic content
    scrollable(container(content).padding(10))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub fn validate_and_create_tunnel(state: &TunnelFormState) -> Result<Tunnel, String> {
    if state.name.trim().is_empty() {
        return Err("Name is required".to_string());
    }

    if state.local_port.trim().is_empty() {
        return Err("Local port is required".to_string());
    }

    if state.remote_host.trim().is_empty() {
        return Err("Remote host is required".to_string());
    }

    if state.remote_port.trim().is_empty() {
        return Err("Remote port is required".to_string());
    }

    if state.ssh_user.trim().is_empty() {
        return Err("SSH user is required".to_string());
    }

    if state.ssh_host.trim().is_empty() {
        return Err("SSH host is required".to_string());
    }

    let web_url_opt = if state.web_url.trim().is_empty() {
        None
    } else {
        let trimmed = state.web_url.trim();
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            Some(trimmed.to_string())
        } else {
            Some(format!("http://{}", trimmed))
        }
    };

    Ok(Tunnel {
        id: uuid::Uuid::new_v4().to_string(),
        name: state.name.to_string(),
        local_host: state.local_host.to_string(),
        local_port: state.local_port.to_string(),
        remote_host: state.remote_host.to_string(),
        remote_port: state.remote_port.to_string(),
        ssh_user: state.ssh_user.to_string(),
        ssh_host: state.ssh_host.to_string(),
        ssh_port: state.ssh_port.to_string(),
        private_key: state.private_key.to_string(),
        web_url: web_url_opt,
    })
}

pub fn browse_for_private_key() -> Option<String> {
    rfd::FileDialog::new()
        .add_filter("SSH Keys", &["pem", "key", "pub", "ppk"])
        .add_filter("All Files", &["*"])
        .set_title("Select SSH Private Key")
        .pick_file()
        .and_then(|path| path.to_str().map(|s| s.to_string()))
}
