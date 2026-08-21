UPDATE books
SET
    title = :title,
    subtitle = :subtitle,
    author = :author,
    isbn = :isbn,
    first_published = :first_published,
    owner = :owner,
    notes = :notes,
    status = :status,
    updated = :updated
WHERE id = :id
