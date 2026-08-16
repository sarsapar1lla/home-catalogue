use crate::gui::App;

mod catalogue;
mod gui;
mod model;

fn main() {
    iced::application(App::default, App::update, App::view)
        .subscription(App::subscription)
        .run()
        .unwrap();
}
