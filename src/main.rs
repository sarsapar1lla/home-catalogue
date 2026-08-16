use jiff::{Timestamp, civil::date};
use uuid::Uuid;

use crate::{
    catalogue::{Cache, Catalogue, DatabaseCatalogue, Searchable},
    model::{Author, Book},
};

mod catalogue;
mod model;

const CREATE: &str = "
    CREATE TABLE books (
        id BLOB PRIMARY KEY,
        title TEXT NOT NULL,
        subtitle TEXT,
        author TEXT NOT NULL,
        isbn TEXT,
        first_published INTEGER,
        status TEXT NOT NULL,
        created TEXT NOT NULL,
        updated TEXT NOT NULL
    );
";

fn main() {
    let connection = rusqlite::Connection::open_in_memory().unwrap();
    connection.execute(CREATE, []).unwrap();

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
                .status(model::Status::Available)
                .created(created)
                .updated(created)
                .build(),
        )
        .unwrap();

    cache
        .update(
            Book::builder()
                .id(id)
                .title("In Cold Blood".to_string())
                .author(Author::Single("Truman Capote".to_string()))
                .isbn("1234".to_string())
                .first_published(1960)
                .status(model::Status::Removed {
                    on: date(2026, 8, 16),
                    reason: Some("Only for tests!".to_string()),
                })
                .created(created)
                .updated(Timestamp::now())
                .build(),
        )
        .unwrap();

    let books = cache
        .search(catalogue::Search::Isbn("1234".to_string()))
        .unwrap();

    for book in books {
        println!("{book:?}");
    }
}
