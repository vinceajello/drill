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
    Test,
    Create,
    Cancel,
}

pub fn view<'a>(
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
    let mut content: Column<'a, Message> = column![
        text("Create New Tunnel").size(20),
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
        text_input("SSH host", ssh_host)
            .on_input(Message::SshHostChanged)
            .padding(8),
        text_input("SSH port", ssh_port)
            .on_input(Message::SshPortChanged)
            .padding(8),
        text_input("Private key path (optional)", private_key)
            .on_input(Message::PrivateKeyChanged)
            .padding(8),
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
    content = content.push(
        row![
            button("Cancel").on_press(Message::Cancel).padding(8),
            text(" "),
            button("Test").on_press(Message::Test).padding(8),
            text(" "),
            button("Create").on_press(Message::Create).padding(8),
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
