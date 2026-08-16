use iced::{
    Element, Subscription,
    keyboard::{self, key},
    widget::{button, column, container, text, text_input},
};
use jiff::Timestamp;
use uuid::Uuid;

use crate::{
    catalogue::{Cache, Catalogue, DatabaseCatalogue, Search, Searchable},
    model::{Author, Book, Status},
};

#[derive(Debug, Clone)]
pub enum Message {
    Home,
    BeginSearch,
    ExecuteSearch(Search),
}

pub struct App {
    screen: Screen,
    cache: Cache,
}

impl App {
    pub fn new() -> Self {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        setup_db(&connection);

        let catalogue = DatabaseCatalogue::new(connection);
        let cache = Cache::new(Box::new(catalogue));
        let id = Uuid::new_v4();
        let created = Timestamp::now();
        cache
            .add(
                Book::builder()
                    .id(id)
                    .title("In Cold Blood".to_string())
                    .author(Author::Single("Truman Capote".to_string()))
                    .isbn("1234".to_string())
                    .first_published(1960)
                    .status(Status::Available)
                    .created(created)
                    .updated(created)
                    .build(),
            )
            .unwrap();

        Self {
            screen: Screen::Home,
            cache,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Screen::Home => container("HomeCat").into(),
            Screen::SearchForm => {
                let isbn = String::new();
                let text_box = text_input("ISBN...", &isbn);
                let submit = button("Search").on_press(Message::ExecuteSearch(Search::Isbn(isbn)));

                column![text_box, submit].into()
            }
            Screen::SearchResult { books, highlighted } => {
                text!("{books:?} - {highlighted}").into()
            }
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Home => self.screen = Screen::Home,
            Message::BeginSearch => self.screen = Screen::SearchForm,
            Message::ExecuteSearch(search) => {
                let books = self.cache.search(search).unwrap();
                self.screen = Screen::SearchResult {
                    books,
                    highlighted: 0,
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        keyboard::listen().filter_map(|event| {
            let keyboard::Event::KeyPressed {
                modified_key,
                repeat: false,
                ..
            } = event
            else {
                return None;
            };

            match modified_key {
                keyboard::Key::Character(c) if c == "s" => Some(Message::BeginSearch),
                _ => None,
            }
        })
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

fn setup_db(connection: &rusqlite::Connection) {
    connection
        .execute(include_str!("./sql/create.sql"), [])
        .unwrap();
}

enum Screen {
    Home,
    SearchForm,
    SearchResult { books: Vec<Book>, highlighted: u16 },
}
