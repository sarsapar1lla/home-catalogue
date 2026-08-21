use iced::{
    Element,
    Length::Fill,
    Subscription,
    keyboard::{self},
    widget::{button, column, container, image, row, text, text_input},
};
use jiff::{Timestamp, tz::TimeZone};
use uuid::Uuid;

use crate::{
    catalogue::{Cache, Catalogue, DatabaseCatalogue, Search, Searchable},
    model::{Author, Book, Status},
};

#[derive(Debug, Clone)]
pub enum KeyPress {
    S,
    Escape,
}

#[derive(Debug, Clone)]
pub enum Message {
    Home,
    SubscribedKeyPress(KeyPress),
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
                    .owner("Tim".into())
                    .notes("This is a really good book!".into())
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
            Message::SubscribedKeyPress(KeyPress::Escape) => self.screen = Screen::Home,
            Message::SubscribedKeyPress(KeyPress::S) if self.screen == Screen::Home => {
                self.screen = Screen::SearchForm {
                    isbn: String::new(),
                }
            }
            Message::SubscribedKeyPress(_) => {}
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
                keyboard::Key::Character(c) if c == "s" => {
                    Some(Message::SubscribedKeyPress(KeyPress::S))
                }
                keyboard::Key::Named(keyboard::key::Named::Escape) => {
                    Some(Message::SubscribedKeyPress(KeyPress::Escape))
                }
                _ => None,
            }
        })
    }

    fn compact_book(&self, book: &Book) -> Element<'_, Message> {
        let author_text = match book.author() {
            Author::Single(author) => author.to_string(),
            Author::Several {
                first,
                second,
                rest,
            } => match &rest[..] {
                [] => format!("{first} and {second}"),
                [one] => format!("{first}, {second}, and {one}"),
                _ => format!("{first}, {second}, et al."),
            },
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
            Author::Several {
                first,
                second,
                rest,
            } => match &rest[..] {
                [] => format!("{first} and {second}"),
                [one] => format!("{first}, {second}, and {one}"),
                _ => format!("{first}, {second}, et al."),
            },
        };
        let title = container(text(book.title().to_string()).size(30).center())
            .align_left(Fill)
            .padding(10);
        let first_published = book
            .first_published()
            .map(|year| text(format!("First published: {year}")));
        let notes = book.notes().map(|notes| text(format!("Notes: {notes}")));
        let info = container(column![
            text(format!("Author(s): {author_text}")),
            first_published,
            text(format!("Owner: {}", book.owner().to_string())),
            notes,
            text(format!("Status: {:?}", book.status())),
            text(format!(
                "Added: {}",
                book.created().to_zoned(TimeZone::system()).date()
            )),
            text(format!(
                "Last updated: {}",
                book.updated()
                    .to_zoned(TimeZone::system())
                    .strftime("%A, %d %B %Y at %H:%M:%S %Z")
            )),
        ])
        .padding(50);
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
