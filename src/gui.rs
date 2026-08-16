use iced::widget::{Container, container};

use crate::catalogue::{Cache, DatabaseCatalogue, Search};

pub enum Message {
    Home,
    BeginSearch,
    ExecuteSearch(Search),
}

pub struct App {
    cache: Cache,
}

impl App {
    pub fn new() -> Self {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        let catalogue = DatabaseCatalogue::new(connection);
        let cache = Cache::new(Box::new(catalogue));
        Self { cache }
    }

    pub fn view(&self) -> Container<Message> {
        container("Hello World!")
    }

    pub fn update(&mut self, message: Message) {}
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
