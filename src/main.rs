mod app;
mod config;
mod logs;
mod panels;
mod systemtray;
mod tunnels;
mod windows;

use app::App;

fn main() -> iced::Result {
    iced::daemon(App::title_fn, App::update_fn, App::view_fn)
        .subscription(App::subscription_fn)
        .run_with(|| {
            let (app, task) = App::new();
            (app, task)
        })
}


