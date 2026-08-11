UPDATE books
SET
    title = :title,
    author = :author,
    isbn = :isbn,
    originally_published = :originally_published,
    edition = :edition,
    edition_published = :edition_published,
    status = :status,
    updated = :updated
WHERE id = :id
