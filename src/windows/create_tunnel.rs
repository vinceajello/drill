use crate::tunnels::Tunnel;
use iced::widget::{button, column, container, row, text, text_input, Column};
use iced::{Element, Length};

#[derive(Debug, Clone)]
pub enum Message {
    NameChanged(String),
    LocalHostChanged(String),
    LocalPortChanged(String),
    RemoteHostChanged(String),
    RemotePortChanged(String),
    SshUserChanged(String),
    SshHostChanged(String),
    SshPortChanged(String),
    PrivateKeyChanged(String),
    BrowsePrivateKey,
    Test,
    Create,
    Cancel,
}

pub fn view<'a>(
    is_edit_mode: bool,
    name: &str,
    local_host: &str,
    local_port: &str,
    remote_host: &str,
    remote_port: &str,
    ssh_user: &str,
    ssh_host: &str,
    ssh_port: &str,
    private_key: &str,
    error_message: &'a Option<String>,
    test_message: &'a Option<String>,
) -> Element<'a, Message> {
    let title = if is_edit_mode { "Edit Tunnel" } else { "Drill New Tunnel" };
    let mut content: Column<'a, Message> = column![
        text(title).size(20),
        text("").size(8),
        text("Tunnel Name:").size(14),
        text_input("Enter tunnel name", name)
            .on_input(Message::NameChanged)
            .padding(8),
        text("").size(4),
        row![
            column![
                text("Local Host").size(12),
                text_input("localhost", local_host)
                    .on_input(Message::LocalHostChanged)
                    .padding(8),
            ]
            .spacing(2)
            .width(Length::Fill),
            text(" ").width(Length::Fixed(10.0)),
            column![
                text("Local Port").size(12),
                text_input("Port (e.g., 8080)", local_port)
                    .on_input(Message::LocalPortChanged)
                    .padding(8),
            ]
            .spacing(2)
            .width(Length::Fill),
        ],
        text("").size(4),
        row![
            column![
                text("Remote Host").size(12),
                text_input("Remote host", remote_host)
                    .on_input(Message::RemoteHostChanged)
                    .padding(8),
            ]
            .spacing(2)
            .width(Length::Fill),
            text(" ").width(Length::Fixed(10.0)),
            column![
                text("Remote Port").size(12),
                text_input("Remote port", remote_port)
                    .on_input(Message::RemotePortChanged)
                    .padding(8),
            ]
            .spacing(2)
            .width(Length::Fill),
        ],
        text("").size(4),
        text("SSH Connection:").size(14),
        text_input("SSH user", ssh_user)
            .on_input(Message::SshUserChanged)
            .padding(8),
        row![
            column![
                text("SSH Host").size(12),
                text_input("SSH host", ssh_host)
                    .on_input(Message::SshHostChanged)
                    .padding(8),
            ]
            .spacing(2)
            .width(Length::Fill),
            text(" ").width(Length::Fixed(10.0)),
            column![
                text("SSH Port").size(12),
                text_input("Port (e.g., 22)", ssh_port)
                    .on_input(Message::SshPortChanged)
                    .padding(8),
            ]
            .spacing(2)
            .width(Length::Fill),
        ],
        text("").size(4),
        text("Private Key (optional)").size(12),
        row![
            text_input("Path to private key file", private_key)
                .on_input(Message::PrivateKeyChanged)
                .padding(8)
                .width(Length::Fill),
            text(" ").width(Length::Fixed(8.0)),
            button("Browse")
                .on_press(Message::BrowsePrivateKey)
                .padding(8),
        ]
        .align_y(iced::Alignment::Center),
    ]
    .spacing(5)
    .padding(20);

    if let Some(error) = error_message {
        content = content.push(text("").size(4));
        content = content.push(
            text(error)
                .color(iced::Color::from_rgb(0.8, 0.0, 0.0))
                .wrapping(iced::widget::text::Wrapping::Word)
        );
    }

    if let Some(test_msg) = test_message {
        content = content.push(text("").size(4));
        if test_msg.starts_with("Success") || test_msg.starts_with("✓") {
            content = content.push(
                text(test_msg)
                    .color(iced::Color::from_rgb(0.0, 0.6, 0.0))
                    .wrapping(iced::widget::text::Wrapping::Word)
            );
        } else {
            content = content.push(
                text(test_msg)
                    .color(iced::Color::from_rgb(0.8, 0.5, 0.0))
                    .wrapping(iced::widget::text::Wrapping::Word)
            );
        }
    }

    content = content.push(text("").size(8));
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

    container(content)
        .width(Length::Fill)
        .height(Length::Shrink)
        .padding(10)
        .into()
}

pub fn validate_and_create_tunnel(
    name: &str,
    local_host: &str,
    local_port: &str,
    remote_host: &str,
    remote_port: &str,
    ssh_user: &str,
    ssh_host: &str,
    ssh_port: &str,
    private_key: &str,
) -> Result<Tunnel, String> {
    if name.trim().is_empty() {
        return Err("Name is required".to_string());
    }

    if local_port.trim().is_empty() {
        return Err("Local port is required".to_string());
    }

    if remote_host.trim().is_empty() {
        return Err("Remote host is required".to_string());
    }

    if remote_port.trim().is_empty() {
        return Err("Remote port is required".to_string());
    }

    if ssh_user.trim().is_empty() {
        return Err("SSH user is required".to_string());
    }

    if ssh_host.trim().is_empty() {
        return Err("SSH host is required".to_string());
    }

    Ok(Tunnel {
        id: uuid::Uuid::new_v4().to_string(),
        name: name.to_string(),
        local_host: local_host.to_string(),
        local_port: local_port.to_string(),
        remote_host: remote_host.to_string(),
        remote_port: remote_port.to_string(),
        ssh_user: ssh_user.to_string(),
        ssh_host: ssh_host.to_string(),
        ssh_port: ssh_port.to_string(),
        private_key: private_key.to_string(),
    })
}

/// Open file picker dialog to select a private key file
/// This function shows hidden files by default
pub fn browse_for_private_key() -> Option<String> {
    rfd::FileDialog::new()
        .add_filter("SSH Keys", &["pem", "key", "pub", "ppk"])
        .add_filter("All Files", &["*"])
        .set_title("Select SSH Private Key")
        .pick_file()
        .and_then(|path| path.to_str().map(|s| s.to_string()))
}
