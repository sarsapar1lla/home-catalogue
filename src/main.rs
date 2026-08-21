use crate::gui::App;

mod catalogue;
mod gui;
mod model;

fn main() {
    iced::application(App::default, App::update, App::view)
        .title("HomeCat")
        .theme(iced::Theme::CatppuccinMocha)
        .subscription(App::subscription)
        .run()
        .unwrap();
}
