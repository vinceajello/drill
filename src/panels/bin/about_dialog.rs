use iced::{
    widget::{button, column, container, text},
    window, Element, Length, Size, Task,
};

fn main() -> iced::Result {
    iced::application("About Drill", AboutDialog::update, AboutDialog::view)
        .window_size(Size::new(400.0, 300.0))
        .run()
}

struct AboutDialog;

#[derive(Debug, Clone)]
enum Message {
    Close,
}

impl Default for AboutDialog {
    fn default() -> Self {
        AboutDialog
    }
}

impl AboutDialog {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Close => window::get_latest().and_then(window::close),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let platform = get_platform();

        let content = column![
            text("Drill").size(24),
            text("Version 0.1.0").size(14),
            text("").size(8),
            text("A multi-platform tunnel drilling application"),
            text("for macOS, Windows, and Linux"),
            text("").size(8),
            text(format!("Platform: {}", platform)),
            text("").size(8),
            text("© 2025").size(12),
            text("").size(16),
            button("OK").on_press(Message::Close),
        ]
        .spacing(10)
        .padding(20)
        .align_x(iced::alignment::Horizontal::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

fn get_platform() -> &'static str {
    #[cfg(target_os = "macos")]
    return "macOS";

    #[cfg(target_os = "windows")]
    return "Windows";

    #[cfg(target_os = "linux")]
    return "Linux";

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    return "Unknown";
}
