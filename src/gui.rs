use iced::{
    Alignment, Element,
    Length::Fill,
    Subscription, Theme,
    keyboard::{self},
    padding,
    widget::{Button, Column, Container, Image, MouseArea, Row, Text, TextInput, container},
};
use jiff::{Timestamp, civil::date, tz::TimeZone};
use uuid::Uuid;

use crate::{
    catalogue::{Cache, Catalogue, DatabaseCatalogue, Search, Searchable},
    model::{Author, Book, Status},
};

const IN_COLD_BLOOD: &[u8] = include_bytes!("../in_cold_blood.jpg");
const BREAKFAST: &[u8] = include_bytes!("../breakfast.jpg");
const PAGE_SIZE: usize = 5;

#[derive(Debug, PartialEq, Clone)]
pub enum Screen {
    Browse,
    Home,
    Search,
}

#[derive(Debug, Clone)]
pub enum Message {
    NavigateTo(Screen),
    SearchInput(String),
    ExecuteSearch,
    BookSelected(usize),
    BookDeselected,
    PageUp,
    PageDown,
}

pub struct App {
    theme: Theme,
    cache: Cache,
    screen: Screen,
    search_input: String,
    browse_state: Option<BrowseState>,
    search_result: Option<SearchResult>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn theme() -> Theme {
        Theme::CatppuccinMocha
    }

    pub fn new() -> Self {
        let connection = rusqlite::Connection::open_in_memory().unwrap();
        setup_db(&connection);

        let catalogue = DatabaseCatalogue::new(connection);
        let cache = Cache::new(Box::new(catalogue));
        cache
            .add(
                Book::builder()
                    .id(Uuid::new_v4())
                    .title("In Cold Blood".to_string())
                    .author(Author::Single("Truman Capote".to_string()))
                    .isbn("1234".to_string())
                    .first_published(1960)
                    .owner("Tim".into())
                    .notes("This is a really good book!".into())
                    .status(Status::Available)
                    .created(Timestamp::now())
                    .updated(Timestamp::now())
                    .build(),
            )
            .unwrap();

        cache
            .add(
                Book::builder()
                    .id(Uuid::new_v4())
                    .title("Breakfast at Tiffany's".to_string())
                    .author(Author::Single("Truman Capote".to_string()))
                    .isbn("5678".to_string())
                    .first_published(1963)
                    .owner("Tim".into())
                    .cover_image(BREAKFAST.to_vec())
                    .status(Status::LoanedOut {
                        on: date(2026, 7, 1),
                        to: "Oz".into(),
                    })
                    .created(Timestamp::now())
                    .updated(Timestamp::now())
                    .build(),
            )
            .unwrap();

        Self {
            theme: App::theme(),
            cache,
            screen: Screen::Home,
            search_input: String::new(),
            browse_state: None,
            search_result: None,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        match &self.screen {
            Screen::Browse => self.browse_screen(),
            Screen::Home => self.home_screen(),
            Screen::Search => self.search_result(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::NavigateTo(Screen::Browse) => {
                let books = self.cache.list().unwrap();
                self.browse_state = Some(BrowseState {
                    books: books,
                    selected: None,
                });
                self.screen = Screen::Browse;
            }
            Message::NavigateTo(screen) => self.screen = screen,
            Message::SearchInput(input) => {
                self.screen = Screen::Search;
                self.search_input = input;

                if self.search_input.is_empty() {
                    self.search_result = None;
                }
            }
            Message::ExecuteSearch => {
                let books_by_isbn = self
                    .cache
                    .search(Search::Isbn(self.search_input.to_owned()))
                    .unwrap();
                let books_by_title = self
                    .cache
                    .search(Search::Title(self.search_input.to_owned()))
                    .unwrap();
                let books_by_author = self
                    .cache
                    .search(Search::Author(self.search_input.to_owned()))
                    .unwrap();

                let books = [books_by_isbn, books_by_title, books_by_author].concat();
                self.search_result = Some(SearchResult {
                    books,
                    page: 0,
                    selected: None,
                });
            }
            Message::BookSelected(index) if self.screen == Screen::Search => {
                if let Some(result) = self.search_result.as_mut() {
                    result.select(index);
                }
            }
            Message::BookDeselected if self.screen == Screen::Search => {
                if let Some(result) = self.search_result.as_mut() {
                    result.deselect();
                }
            }
            Message::BookSelected(index) => {
                if let Some(state) = self.browse_state.as_mut() {
                    state.select(index);
                }
            }
            Message::BookDeselected => {
                if let Some(state) = self.browse_state.as_mut() {
                    state.deselect();
                }
            }
            Message::PageUp => {
                if let Some(state) = self.search_result.as_mut() {
                    state.page_up();
                }
            }
            Message::PageDown => {
                if let Some(state) = self.search_result.as_mut() {
                    state.page_down();
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
                keyboard::Key::Named(keyboard::key::Named::Escape) => {
                    Some(Message::NavigateTo(Screen::Home))
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
        Container::new(
            Row::new()
                .push(Text::new(book.title().to_string()).align_y(Alignment::Center))
                .push(Text::new(" | ").align_y(Alignment::Center))
                .push(Text::new(author_text).align_y(Alignment::Center))
                .push(iced::widget::space::horizontal())
                .push(self.status_box(book.status())),
        )
        .padding(10)
        .align_left(Fill)
        .style(container::bordered_box)
        .into()
    }

    fn status_box(&self, status: &Status) -> Element<'_, Message> {
        let (status_text, colour) = match status {
            Status::Available | Status::LoanedIn { on: _, from: _ } => (
                "Available",
                self.theme.extended_palette().success.weak.color,
            ),
            Status::LoanedOut { on: _, to: _ } => {
                ("On Loan", self.theme.extended_palette().warning.weak.color)
            }
            Status::Removed { on: _, reason: _ } => {
                ("Removed", self.theme.extended_palette().danger.weak.color)
            }
        };
        Container::new(
            Text::new(status_text)
                .center()
                .color(self.theme.palette().text.inverse()),
        )
        .padding(padding::horizontal(5).vertical(2))
        .style(move |theme| container::bordered_box(theme).background(colour))
        .into()
    }

    fn selected_book(&self, book: &Book) -> Element<'_, Message> {
        let author = Text::new(match book.author() {
            Author::Single(author) => format!("Author: {}", author),
            Author::Several {
                first,
                second,
                rest,
            } => match &rest[..] {
                [] => format!("Authors: {first} and {second}"),
                [one] => format!("Authors: {first}, {second}, and {one}"),
                _ => format!("Authors: {first}, {second}, et al."),
            },
        });

        let title = Container::new(Text::new(book.title().to_string()).size(30).center())
            .align_left(Fill)
            .padding(padding::bottom(10));

        let first_published = book
            .first_published()
            .map(|year| Text::new(format!("First published: {year}")));

        let notes = book
            .notes()
            .map(|notes| Text::new(format!("Notes: {notes}")));

        let status = Text::new(format!(
            "Status: {}",
            match book.status() {
                Status::Available => "Available".to_string(),
                Status::LoanedIn { on, from } => format!("Loaned from {from} on {on}"),
                Status::LoanedOut { on, to } => format!("Loaned to {to} on {on}"),
                Status::Removed {
                    on,
                    reason: Some(reason),
                } => {
                    format!("Removed on {on}: {reason}")
                }
                Status::Removed { on, reason: _ } => format!("Removed on {on}"),
            }
        ));

        let info = Container::new(
            Column::new()
                .push(author)
                .push(first_published)
                .push(Text::new(format!("Owner: {}", book.owner())))
                .push(notes)
                .push(status)
                .push("\n")
                .push(Text::new(format!(
                    "Added: {}",
                    book.created().to_zoned(TimeZone::system()).date()
                )))
                .push(Text::new(format!(
                    "Last updated: {}",
                    book.updated()
                        .to_zoned(TimeZone::system())
                        .strftime("%A, %d %B %Y at %H:%M:%S %Z")
                ))),
        )
        .padding(50)
        .align_y(Alignment::Center);

        let cover_image = book
            .cover_image()
            .map(|bytes| Image::new(iced::widget::image::Handle::from_bytes(bytes.to_vec())))
            .unwrap_or_else(|| Image::new(iced::widget::image::Handle::from_bytes(IN_COLD_BLOOD)));

        Container::new(
            Column::new()
                .push(title)
                .push(Row::new().push(cover_image).push(info)),
        )
        .padding(20)
        .align_left(Fill)
        .style(container::bordered_box)
        .into()
    }

    fn books(
        &self,
        books: &[Book],
        page: usize,
        highlighted: Option<&usize>,
    ) -> Element<'_, Message> {
        let page_start = page * PAGE_SIZE;
        let page_end = books.len().min(page_start + PAGE_SIZE);
        let books = books.get(page_start..page_end).expect("Should be correct");
        Container::new(
            Column::new().extend(books.iter().enumerate().map(|(idx, book)| {
                if let Some(picked) = highlighted
                    && idx == *picked
                {
                    MouseArea::new(self.selected_book(book))
                        .on_release(Message::BookDeselected)
                        .into()
                } else {
                    MouseArea::new(self.compact_book(book))
                        .on_release(Message::BookSelected(idx))
                        .into()
                }
            })),
        )
        .style(container::rounded_box)
        .into()
    }

    fn home_screen(&self) -> Element<'_, Message> {
        let buttons = Row::new()
            .push(Button::new("Search").on_press(Message::NavigateTo(Screen::Search)))
            .push(Button::new("Browse").on_press(Message::NavigateTo(Screen::Browse)))
            .spacing(20);
        Container::new(
            Column::new()
                .push(Container::new(Text::new("HomeCat").size(40).center()).center(Fill))
                .push(Container::new(buttons).center(Fill))
                .push(
                    Container::new(Text::new(format!("v{}", env!("CARGO_PKG_VERSION"))).size(12))
                        .align_bottom(Fill)
                        .align_right(Fill),
                ),
        )
        .padding(100)
        .center(Fill)
        .into()
    }

    fn browse_screen(&self) -> Element<'_, Message> {
        let results = match &self.browse_state {
            None => None,
            Some(BrowseState { books, selected: _ }) if books.is_empty() => None,
            Some(BrowseState {
                books,
                selected: highlighted,
            }) => Some(self.books(books, 0, highlighted.as_ref())),
        };
        let book_count = match &self.browse_state {
            None => self.book_count(&[]),
            Some(BrowseState { books, selected: _ }) => self.book_count(books),
        };
        Container::new(Column::new().push(book_count).push(results))
            .padding(100)
            .center_x(Fill)
            .into()
    }

    fn search_bar(&self) -> Element<'_, Message> {
        Container::new(
            TextInput::new("What are you searching for?...", &self.search_input)
                .on_input(Message::SearchInput)
                .on_submit(Message::ExecuteSearch),
        )
        .padding(padding::bottom(20))
        .center_x(Fill)
        .into()
    }

    fn search_result(&self) -> Element<'_, Message> {
        let (results, book_count, page_count) = match &self.search_result {
            None => (
                None,
                self.book_count(&[]),
                Text::new(format!("Page 1 of 1")),
            ),
            Some(result) => (
                if result.books.is_empty() {
                    None
                } else {
                    Some(self.books(&result.books, result.page, result.selected.as_ref()))
                },
                self.book_count(&result.books),
                Text::new(format!(
                    "Page {} of {}",
                    result.page + 1,
                    result.max_page() + 1
                )),
            ),
        };
        let page_buttons = Row::new()
            .push(iced::widget::space::horizontal())
            .push(Button::new("Prev").on_press(Message::PageDown))
            .push(Button::new("Next").on_press(Message::PageUp))
            .spacing(10);
        Container::new(
            Column::new()
                .push(book_count)
                .push(self.search_bar())
                .push(results)
                .push(iced::widget::space::vertical())
                .push(
                    Container::new(page_count)
                        .align_right(Fill)
                        .padding(padding::vertical(5)),
                )
                .push(page_buttons),
        )
        .padding(100)
        .center_x(Fill)
        .into()
    }

    fn book_count(&self, books: &[Book]) -> Element<'_, Message> {
        let count = if books.len() > 0 {
            books.len().to_string()
        } else {
            String::from("No")
        };
        Container::new(Text::new(format!("{} results found", count)))
            .padding(5)
            .align_right(Fill)
            .into()
    }
}

fn setup_db(connection: &rusqlite::Connection) {
    connection
        .execute(include_str!("./sql/create.sql"), [])
        .unwrap();
}

struct SearchResult {
    books: Vec<Book>,
    page: usize,
    selected: Option<usize>,
}

impl SearchResult {
    fn page_up(&mut self) {
        if self.page < self.max_page() && self.max_page() > 0 {
            self.page += 1;
        }
    }

    fn page_down(&mut self) {
        if self.page > 0 {
            self.page -= 1;
        }
    }

    fn select(&mut self, index: usize) {
        self.selected = Some(index);
    }

    fn deselect(&mut self) {
        self.selected = None;
    }

    fn max_page(&self) -> usize {
        self.books.len().div_euclid(PAGE_SIZE)
    }
}

struct BrowseState {
    books: Vec<Book>,
    selected: Option<usize>,
}

impl BrowseState {
    fn select(&mut self, index: usize) {
        self.selected = Some(index);
    }

    fn deselect(&mut self) {
        self.selected = None;
    }
}
