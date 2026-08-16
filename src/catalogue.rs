use std::sync::Mutex;

use crate::model::{Author, Book, Status};

pub enum Search {
    Isbn(String),
    Title(String),
    Author(String),
}

pub trait Searchable {
    fn search(&self, search: Search) -> Result<Vec<Book>, &'static str>;
}

pub trait Catalogue {
    fn add(&self, book: Book) -> Result<(), &'static str>;
    fn update(&self, book: Book) -> Result<(), &'static str>;
    fn list(&self) -> Result<Vec<Book>, &'static str>;
}

pub struct Cache {
    cache: Mutex<Vec<Book>>,
    delegate: Box<dyn Catalogue>,
}

impl Searchable for Cache {
    fn search(&self, search: Search) -> Result<Vec<Book>, &'static str> {
        match search {
            Search::Isbn(isbn) => Ok(self
                .cache
                .lock()
                .expect("Can lock mutex")
                .iter()
                .filter(|book| book.isbn().is_some_and(|it| it == isbn))
                .cloned()
                .collect()),
            Search::Title(title) => Ok(self
                .cache
                .lock()
                .expect("Can lock mutex")
                .iter()
                .filter(|book| book.title() == title)
                .cloned()
                .collect()),
            Search::Author(author) => Ok(self
                .cache
                .lock()
                .expect("Can lock mutex")
                .iter()
                .filter(|book| match book.author() {
                    Author::Single(single_author) => single_author == &author,
                    Author::Several(authors) => authors.contains(&author),
                })
                .cloned()
                .collect()),
        }
    }
}

impl Catalogue for Cache {
    fn add(&self, book: Book) -> Result<(), &'static str> {
        self.delegate.add(book)?;
        self.refresh()
    }

    fn update(&self, book: Book) -> Result<(), &'static str> {
        self.delegate.update(book)?;
        self.refresh()
    }

    fn list(&self) -> Result<Vec<Book>, &'static str> {
        Ok(self.cache.lock().expect("Can lock mutex").to_vec())
    }
}

impl Cache {
    pub fn new(delegate: Box<dyn Catalogue>) -> Self {
        let cache = delegate.list().expect("Catalogue available");
        Self {
            cache: Mutex::new(cache),
            delegate,
        }
    }

    fn refresh(&self) -> Result<(), &'static str> {
        let mut books = self.delegate.list()?;
        let mut cache = self.cache.lock().expect("Can lock mutex");
        std::mem::swap(&mut *cache, &mut books);
        Ok(())
    }
}

pub struct DatabaseCatalogue {
    connection: rusqlite::Connection,
}

impl Catalogue for DatabaseCatalogue {
    fn add(&self, book: Book) -> Result<(), &'static str> {
        self.connection
            .execute(
                include_str!("./sql/add.sql"),
                rusqlite::named_params! {
                    ":id": book.id(),
                    ":title": book.title(),
                    ":subtitle": book.subtitle(),
                    ":author": book.author(),
                    ":isbn": book.isbn(),
                    ":first_published": book.first_published(),
                    ":status": book.status(),
                    ":created": book.created(),
                    ":updated": book.updated(),
                },
            )
            .unwrap();

        Ok(())
    }

    fn update(&self, book: Book) -> Result<(), &'static str> {
        self.connection
            .execute(
                include_str!("./sql/update.sql"),
                rusqlite::named_params! {
                    ":id": book.id(),
                    ":title": book.title(),
                    ":subtitle": book.subtitle(),
                    ":author": book.author(),
                    ":isbn": book.isbn(),
                    ":first_published": book.first_published(),
                    ":status": book.status(),
                    ":updated": book.updated(),
                },
            )
            .unwrap();

        Ok(())
    }

    fn list(&self) -> Result<Vec<Book>, &'static str> {
        let mut statement = self.connection.prepare("SELECT * FROM books").unwrap();
        statement
            .query_map([], |row| self.book_from(row))
            .unwrap()
            .map(|row| row.map_err(|_| "Failed to contruct Book from row data"))
            .collect()
    }
}

impl DatabaseCatalogue {
    pub fn new(connection: rusqlite::Connection) -> Self {
        Self { connection }
    }

    fn book_from(&self, row: &rusqlite::Row) -> Result<Book, rusqlite::Error> {
        Ok(Book::builder()
            .id(row.get(0)?)
            .title(row.get(1)?)
            .maybe_subtitle(row.get(2)?)
            .author(row.get(3)?)
            .maybe_isbn(row.get(4)?)
            .maybe_first_published(row.get(5)?)
            .status(row.get(6)?)
            .created(row.get(7)?)
            .updated(row.get(8)?)
            .build())
    }
}

impl rusqlite::ToSql for Status {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        match serde_json::to_string(self) {
            Ok(json) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Text(json),
            )),
            Err(err) => Err(rusqlite::Error::ToSqlConversionFailure(err.into())),
        }
    }
}

impl rusqlite::types::FromSql for Status {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Text(json) => match serde_json::from_slice(json) {
                Ok(status) => Ok(status),
                Err(_) => Err(rusqlite::types::FromSqlError::InvalidType),
            },
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

impl rusqlite::ToSql for Author {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        match serde_json::to_string(self) {
            Ok(json) => Ok(rusqlite::types::ToSqlOutput::Owned(
                rusqlite::types::Value::Text(json),
            )),
            Err(err) => Err(rusqlite::Error::ToSqlConversionFailure(err.into())),
        }
    }
}

impl rusqlite::types::FromSql for Author {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        match value {
            rusqlite::types::ValueRef::Text(json) => match serde_json::from_slice(json) {
                Ok(status) => Ok(status),
                Err(_) => Err(rusqlite::types::FromSqlError::InvalidType),
            },
            _ => Err(rusqlite::types::FromSqlError::InvalidType),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod cache_tests {
        use super::*;

        mod search_tests {
            use std::str::FromStr;

            use uuid::Uuid;

            use super::*;

            #[test]
            fn returns_books_with_matching_isbn() {
                let books = vec![
                    book(None, None, Some("1234")),
                    book(Some("Feersum Endjinn"), Some("Iain M. Banks"), Some("5678")),
                    book(Some("Orlando"), Some("Virginia Woolfe"), None),
                ];
                let delegate = InMemoryCatalogue {
                    books: Mutex::new(books),
                };
                let cache = Cache::new(Box::new(delegate));

                let actual = cache.search(Search::Isbn("1234".to_string())).unwrap();
                assert_eq!(actual, vec![book(None, None, Some("1234"))])
            }

            #[test]
            fn returns_books_with_matching_title() {
                let books = vec![
                    book(None, None, Some("1234")),
                    book(Some("Feersum Endjinn"), Some("Iain M. Banks"), Some("5678")),
                    book(Some("Orlando"), Some("Virginia Woolfe"), None),
                ];
                let delegate = InMemoryCatalogue {
                    books: Mutex::new(books),
                };
                let cache = Cache::new(Box::new(delegate));

                let actual = cache
                    .search(Search::Title("In Cold Blood".to_string()))
                    .unwrap();
                assert_eq!(actual, vec![book(None, None, Some("1234"))])
            }

            #[test]
            fn returns_books_with_matching_author() {
                let books = vec![
                    book(None, None, Some("1234")),
                    book(Some("Feersum Endjinn"), Some("Iain M. Banks"), Some("5678")),
                    book(Some("Orlando"), Some("Virginia Woolfe"), None),
                ];
                let delegate = InMemoryCatalogue {
                    books: Mutex::new(books),
                };
                let cache = Cache::new(Box::new(delegate));

                let actual = cache
                    .search(Search::Author("Truman Capote".to_string()))
                    .unwrap();
                assert_eq!(actual, vec![book(None, None, Some("1234"))])
            }

            fn book(title: Option<&str>, author: Option<&str>, isbn: Option<&str>) -> Book {
                Book::builder()
                    .id(Uuid::from_str("955ed41d-9411-45c7-91b7-c8c11abbf24e").unwrap())
                    .title(title.unwrap_or("In Cold Blood").to_string())
                    .author(Author::Single(
                        author.unwrap_or("Truman Capote").to_string(),
                    ))
                    .maybe_isbn(isbn.map(|x| x.to_string()))
                    .status(Status::Available)
                    .created("2026-08-11T20:50:00Z".parse().unwrap())
                    .updated("2026-08-11T20:50:00Z".parse().unwrap())
                    .build()
            }
        }

        struct InMemoryCatalogue {
            books: Mutex<Vec<Book>>,
        }

        impl Catalogue for InMemoryCatalogue {
            fn add(&self, book: Book) -> Result<(), &'static str> {
                Ok(self.books.lock().unwrap().push(book))
            }

            fn update(&self, book: Book) -> Result<(), &'static str> {
                let index = self
                    .books
                    .lock()
                    .unwrap()
                    .iter()
                    .position(|b| *b == book)
                    .unwrap();
                self.books.lock().unwrap()[index] = book;
                Ok(())
            }

            fn list(&self) -> Result<Vec<Book>, &'static str> {
                Ok(self.books.lock().unwrap().to_vec())
            }
        }
    }
}
