use uuid::Uuid;

use crate::model::{Book, Status};

pub enum Search {
    Isbn(String),
    Title(String),
    Author(String),
}

pub trait Catalogue {
    fn add(&self, book: Book) -> Result<Uuid, &'static str>;
    fn update(&self, id: Uuid, status: Status) -> Result<(), &'static str>;
    fn search(&self, search: Search) -> Result<Vec<Book>, &'static str>;
}

pub struct DatabaseCatalogue {
    conn: rusqlite::Connection,
}

impl Catalogue for DatabaseCatalogue {
    fn add(&self, book: Book) -> Result<Uuid, &'static str> {
        self.conn
            .execute(
                include_str!("./sql/add.sql"),
                rusqlite::params![
                    book.id(),
                    book.title(),
                    book.author(),
                    book.isbn(),
                    book.originally_published(),
                    book.edition(),
                    book.edition_published(),
                    book.status(),
                    book.created(),
                    book.updated(),
                ],
            )
            .unwrap();

        Ok(book.id().to_owned())
    }

    fn update(&self, id: Uuid, status: Status) -> Result<(), &'static str> {
        todo!()
    }

    fn search(&self, search: Search) -> Result<Vec<Book>, &'static str> {
        let (query, param) = match search {
            Search::Isbn(isbn) => ("SELECT * FROM books WHERE isbn = '?1'", isbn),
            Search::Title(title) => ("SELECT * FROM books WHERE title = '?1'", title),
            _ => todo!(),
        };
        let mut statement = self.conn.prepare(query).unwrap();
        statement
            .query_map([param], |row| Ok(self.book_from(row)))
            .unwrap()
            .into_iter()
            .map(|row| row.map_err(|err| "Error!"))
            .collect()
    }
}

impl DatabaseCatalogue {
    fn book_from(&self, row: &rusqlite::Row) -> Book {
        Book::builder()
            .id(row.get_unwrap(0))
            .title(row.get_unwrap(1))
            .author(row.get_unwrap(2))
            .maybe_isbn(row.get_unwrap(3))
            .maybe_originally_published(row.get_unwrap(4))
            .maybe_edition(row.get_unwrap(5))
            .maybe_edition_published(row.get_unwrap(6))
            .status(row.get_unwrap(7))
            .created(row.get_unwrap(8))
            .updated(row.get_unwrap(9))
            .build()
    }
}

impl rusqlite::ToSql for Status {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        let value = match self {
            Status::Available => "available",
            Status::Loaned { to } => &format!("Loaned to '{to}'"),
            Status::Removed { on, reason } => "Removed",
        };
        Ok(rusqlite::types::ToSqlOutput::Owned(
            rusqlite::types::Value::Text(value.to_string()),
        ))
    }
}

impl rusqlite::types::FromSql for Status {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        let status = match value {
            rusqlite::types::ValueRef::Text(text) => Status::Available,
            _ => panic!(),
        };
        rusqlite::types::FromSqlResult::Ok(status)
    }
}
