use iced::Size;

use crate::gui::App;

mod catalogue;
mod gui;
mod model;

fn main() {
    iced::application(App::default, App::update, App::view)
        .title("HomeCat")
        .window_size(Size::new(1280.0, 720.0))
        .centered()
        .resizable(false)
        .default_font(iced::Font {
            family: iced::font::Family::Monospace,
            ..Default::default()
        })
        .theme(iced::Theme::CatppuccinMocha)
        .subscription(App::subscription)
        .run()
        .unwrap();
}
