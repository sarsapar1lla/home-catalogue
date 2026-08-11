use crate::catalogue::{Cache, DatabaseCatalogue, Searchable};

mod catalogue;
mod model;

const CREATE: &str = "
    BEGIN;
    CREATE TABLE books (
        id TEXT PRIMARY KEY,
        title TEXT NOT NULL,
        author TEXT NOT NULL,
        isbn TEXT,
        originally_published INTEGER,
        edition TEXT,
        edition_published INTEGER,
        status TEXT NOT NULL,
        created TEXT NOT NULL,
        updated TEXT NOT NULL
    );

    INSERT INTO books VALUES (
        '955ed41d-9411-45c7-91b7-c8c11abbf24e',
        'In Cold Blood',
        'Truman Capote',
        '1234',
        1960,
        NULL,
        NULL,
        'something',
        '2026-08-11T20:05:00Z',
        '2026-08-11T20:05:00Z'
    );
    END;
";

fn main() {
    let connection = rusqlite::Connection::open_in_memory().unwrap();
    connection.execute_batch(CREATE).unwrap();

    let catalogue = DatabaseCatalogue::new(connection);
    let cache = Cache::new(Box::new(catalogue));

    let books = cache
        .search(catalogue::Search::Isbn("1234".to_string()))
        .unwrap();

    for book in books {
        println!("{book:?}");
    }
}
