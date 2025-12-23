use iced::widget::{column, container, text, Column};
use iced::{Center, Element, Length};

#[derive(Debug, Clone)]
pub enum Message {
}

pub fn view<'a>() -> Element<'a, Message> {

    let content: Column<'a, Message> = column![
        text("Drill").size(24),
        text("Version 0.1.0").size(14),
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
