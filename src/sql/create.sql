CREATE TABLE books (
    id BLOB PRIMARY KEY,
    title TEXT NOT NULL,
    subtitle TEXT,
    author TEXT NOT NULL,
    isbn TEXT,
    first_published INTEGER,
    owner TEXT NOT NULL,
    notes TEXT,
    cover_image BLOB,
    status TEXT NOT NULL,
    created TEXT NOT NULL,
    updated TEXT NOT NULL
)
