use iced::{
    Element,
    Length::Fill,
    Subscription,
    keyboard::{self},
    widget::{button, column, container, image, row, text, text_input},
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
    SearchFormInput(String),
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
            Screen::Home => self.home_screen(),
            Screen::SearchForm { isbn } => {
                let text_box = text_input("ISBN...", isbn).on_input(Message::SearchFormInput);
                let submit =
                    button("Search").on_press(Message::ExecuteSearch(Search::Isbn(isbn.clone())));

                column![text_box, submit].into()
            }
            Screen::SearchResult { books, highlighted } => {
                container(column(books.into_iter().enumerate().into_iter().map(
                    |(idx, book)| {
                        if idx == (*highlighted as usize) {
                            self.highlighted_book(book)
                        } else {
                            self.compact_book(book)
                        }
                    },
                )))
                .padding(10)
                .style(container::rounded_box)
                .into()
            }
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::Home => self.screen = Screen::Home,
            Message::BeginSearch if self.screen == Screen::Home => {
                self.screen = Screen::SearchForm {
                    isbn: String::new(),
                }
            }
            Message::BeginSearch => {}
            Message::SearchFormInput(input) => self.screen = Screen::SearchForm { isbn: input },
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
                keyboard::Key::Named(keyboard::key::Named::Escape) => Some(Message::Home),
                _ => None,
            }
        })
    }

    fn compact_book(&self, book: &Book) -> Element<'_, Message> {
        let author_text = match book.author() {
            Author::Single(author) => author.to_string(),
            Author::Several(authors) => authors.join(" and "),
        };
        container(row![
            text(book.title().to_string()),
            text(" by "),
            text(author_text)
        ])
        .padding(10)
        .align_left(Fill)
        .style(container::rounded_box)
        .into()
    }

    fn highlighted_book(&self, book: &Book) -> Element<'_, Message> {
        let author_text = match book.author() {
            Author::Single(author) => author.to_string(),
            Author::Several(authors) => authors.join(" and "),
        };
        let title = container(text(book.title().to_string()).size(30).center())
            .align_left(Fill)
            .padding(10);
        let info = container(column![
            text(format!("Author(s): {author_text}")),
            text(format!(
                "First published: {}",
                book.first_published()
                    .map_or("Unknown".to_string(), |year| year.to_string())
            )),
            text(format!("Status: {:?}", book.status()))
        ])
        .padding(20);
        container(column![title, row![image("in_cold_blood.jpg"), info]])
            .padding(20)
            .align_left(Fill)
            .style(|theme: &iced::Theme| {
                let palette = theme.palette();
                container::Style {
                    text_color: Some(palette.text),
                    background: Some(palette.background.into()),
                    border: iced::border::rounded(20),
                    ..Default::default()
                }
            })
            .into()
    }

    fn home_screen(&self) -> Element<'_, Message> {
        container(column![
            image("nano.jpg"),
            container(text("HomeCat").size(40).center()).center(Fill),
            container(text(format!("v{}", env!("CARGO_PKG_VERSION"))).size(12))
                .align_bottom(Fill)
                .align_right(Fill)
        ])
        .padding(100)
        .center(Fill)
        .into()
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

#[derive(Debug, PartialEq)]
enum Screen {
    Home,
    SearchForm { isbn: String },
    SearchResult { books: Vec<Book>, highlighted: u16 },
}
