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
)
