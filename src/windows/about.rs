use iced::widget::{column, container, text, Column, Image};
use iced::{Center, Element, Length};

#[derive(Debug, Clone)]
pub enum Message {
}

pub fn view<'a>() -> Element<'a, Message> {
    // Load the icon
    let icon_bytes = include_bytes!("../../resources/icon.png");
    let icon_handle = iced::widget::image::Handle::from_bytes(icon_bytes.as_slice());

    let content: Column<'a, Message> = column![
        Image::new(icon_handle).width(64).height(64),
        text("").size(4),
        text("Drill").size(24),
        text("Version 0.1.1").size(14),
        text("").size(8),
        text("A multi-platform tunnel drilling application"),
        text("for macOS, Windows, and Linux"),
        text("").size(8),
        text("enjoy drill :)").size(12),
        text("").size(16)
    ]
    .spacing(10)
    .padding(20)
    .align_x(Center);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
