#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod error;
mod logs;
mod notifications;
mod platform;
mod systemtray;
mod tunnels;
mod windows;

use app::App;

fn main() -> iced::Result {
    notifications::init_notifications();

    iced::daemon(App::title_fn, App::update_fn, App::view_fn)
        .subscription(App::subscription_fn)
        .run_with(|| {
            let (app, task) = App::new();
            (app, task)
        })
}
